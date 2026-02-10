use anyhow::Context;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Node, Parser};

use crate::indexer::languages::{
    ExtractedEdge, ExtractedReference, ExtractedSymbol, ExtractionUnit, LanguageAdapter, SymbolKey,
};

pub struct PythonLanguageAdapter;

#[derive(Debug, Clone)]
struct ImportCallHint {
    import_path: String,
    imported_symbol: String,
}

fn scoped_symbol_key(file_path: &str, language: &str, symbol: &str) -> SymbolKey {
    SymbolKey {
        symbol: symbol.to_string(),
        qualified_symbol: Some(format!("{language}:{file_path}::{symbol}")),
        file_path: Some(file_path.to_string()),
        language: Some(language.to_string()),
    }
}

fn language_symbol_key(symbol: &str, language: &str) -> SymbolKey {
    SymbolKey {
        symbol: symbol.to_string(),
        qualified_symbol: None,
        file_path: None,
        language: Some(language.to_string()),
    }
}

impl LanguageAdapter for PythonLanguageAdapter {
    fn language_id(&self) -> &'static str {
        "python"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["py"]
    }

    fn extract(&self, file_path: &str, source: &str) -> anyhow::Result<ExtractionUnit> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .context("failed to load tree-sitter python language")?;

        let tree = parser
            .parse(source, None)
            .context("failed to parse python source")?;

        let language = self.language_id().to_string();
        let import_target_hints = import_target_hints(file_path, source);
        let import_call_hints = import_call_hints(file_path, source);
        let mut symbols = Vec::new();
        let mut references = Vec::new();
        let mut edges = Vec::new();
        let mut stack = vec![tree.root_node()];

        while let Some(node) = stack.pop() {
            match node.kind() {
                "class_definition" => {
                    push_named_definition(
                        node,
                        source,
                        "class",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "function_definition" => {
                    let container = enclosing_class_name(node, source);
                    let kind = if container.is_some() {
                        "method"
                    } else {
                        "function"
                    };
                    if let Some(symbol) = push_named_definition(
                        node,
                        source,
                        kind,
                        container.clone(),
                        file_path,
                        &language,
                        &mut symbols,
                    ) && let Some(container_symbol) = container
                    {
                        edges.push(ExtractedEdge {
                            from_symbol_key: scoped_symbol_key(
                                file_path,
                                &language,
                                &container_symbol,
                            ),
                            to_symbol_key: scoped_symbol_key(file_path, &language, &symbol),
                            edge_kind: "contains".to_string(),
                            confidence: 1.0,
                            provenance: "ast_definition".to_string(),
                        });
                    }
                }
                "call" => {
                    let caller = enclosing_callable_name(node, source);
                    if let Some(function_node) = node.child_by_field_name("function") {
                        collect_call_symbols(
                            function_node,
                            source,
                            caller.as_deref(),
                            file_path,
                            &language,
                            &import_target_hints,
                            &import_call_hints,
                            &mut references,
                            &mut edges,
                        );
                    }
                }
                "import_statement" | "import_from_statement" => {
                    for binding in import_bindings(node, source) {
                        references.push(ExtractedReference {
                            symbol: binding.imported_symbol.clone(),
                            line: binding.start_line,
                            column: binding.start_column,
                        });
                        symbols.push(ExtractedSymbol {
                            symbol: binding.local_symbol.clone(),
                            qualified_symbol: Some(format!(
                                "{language}:{file_path}::{}",
                                binding.local_symbol
                            )),
                            kind: "import".to_string(),
                            language: language.clone(),
                            container: None,
                            start_line: binding.start_line,
                            start_column: binding.start_column,
                            end_line: binding.end_line,
                            end_column: binding.end_column,
                            signature: Some(format!("import {}", binding.local_symbol)),
                        });
                        edges.push(ExtractedEdge {
                            from_symbol_key: scoped_symbol_key(
                                file_path,
                                &language,
                                &binding.local_symbol,
                            ),
                            to_symbol_key: language_symbol_key(&binding.imported_symbol, &language),
                            edge_kind: "imports".to_string(),
                            confidence: 0.9,
                            provenance: "import_resolution".to_string(),
                        });
                    }
                }
                _ => {}
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        symbols.extend(module_constants(file_path, &language, source));

        symbols.sort_by(|left, right| {
            left.start_line
                .cmp(&right.start_line)
                .then(left.start_column.cmp(&right.start_column))
                .then(left.symbol.cmp(&right.symbol))
                .then(left.kind.cmp(&right.kind))
        });
        symbols.dedup_by(|left, right| {
            (
                left.symbol.as_str(),
                left.kind.as_str(),
                left.start_line,
                left.start_column,
            ) == (
                right.symbol.as_str(),
                right.kind.as_str(),
                right.start_line,
                right.start_column,
            )
        });

        references.sort_by(|left, right| {
            left.line
                .cmp(&right.line)
                .then(left.column.cmp(&right.column))
                .then(left.symbol.cmp(&right.symbol))
        });
        references.dedup_by(|left, right| {
            left.symbol == right.symbol && left.line == right.line && left.column == right.column
        });

        edges.sort_by(|left, right| {
            left.from_symbol_key
                .symbol
                .cmp(&right.from_symbol_key.symbol)
                .then(
                    left.from_symbol_key
                        .qualified_symbol
                        .cmp(&right.from_symbol_key.qualified_symbol),
                )
                .then(left.to_symbol_key.symbol.cmp(&right.to_symbol_key.symbol))
                .then(
                    left.to_symbol_key
                        .qualified_symbol
                        .cmp(&right.to_symbol_key.qualified_symbol),
                )
                .then(left.edge_kind.cmp(&right.edge_kind))
        });
        edges.dedup_by(|left, right| {
            left.from_symbol_key.symbol == right.from_symbol_key.symbol
                && left.from_symbol_key.qualified_symbol == right.from_symbol_key.qualified_symbol
                && left.to_symbol_key.symbol == right.to_symbol_key.symbol
                && left.to_symbol_key.qualified_symbol == right.to_symbol_key.qualified_symbol
                && left.edge_kind == right.edge_kind
        });

        Ok(ExtractionUnit {
            symbols,
            references,
            edges,
        })
    }
}

fn push_named_definition(
    node: Node<'_>,
    source: &str,
    kind: &str,
    container: Option<String>,
    file_path: &str,
    language: &str,
    output: &mut Vec<ExtractedSymbol>,
) -> Option<String> {
    let name_node = node.child_by_field_name("name")?;
    if name_node.kind() != "identifier" {
        return None;
    }

    let symbol = node_text(name_node, source)?;
    let (start_line, start_column) = start_position(name_node);
    let (end_line, end_column) = end_position(node);
    output.push(ExtractedSymbol {
        symbol: symbol.clone(),
        qualified_symbol: Some(format!("{language}:{file_path}::{symbol}")),
        kind: kind.to_string(),
        language: language.to_string(),
        container,
        start_line,
        start_column,
        end_line,
        end_column,
        signature: signature_summary(node, source),
    });
    Some(symbol)
}

#[allow(clippy::too_many_arguments)]
fn collect_call_symbols(
    node: Node<'_>,
    source: &str,
    caller: Option<&str>,
    file_path: &str,
    language: &str,
    import_target_hints: &HashMap<String, String>,
    import_call_hints: &HashMap<String, ImportCallHint>,
    references: &mut Vec<ExtractedReference>,
    edges: &mut Vec<ExtractedEdge>,
) {
    match node.kind() {
        "identifier" => {
            let symbol = node_text(node, source).unwrap_or_default();
            if !symbol.is_empty() {
                let (line, column) = start_position(node);
                references.push(ExtractedReference {
                    symbol: symbol.clone(),
                    line,
                    column,
                });
                if let Some(caller_symbol) = caller
                    && let Some(call_hint) = import_call_hints.get(&symbol)
                {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                        to_symbol_key: SymbolKey {
                            symbol: call_hint.imported_symbol.clone(),
                            qualified_symbol: Some(format!(
                                "{language}:{}::{}",
                                call_hint.import_path, call_hint.imported_symbol
                            )),
                            file_path: Some(call_hint.import_path.clone()),
                            language: Some(language.to_string()),
                        },
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                } else if let Some(caller_symbol) = caller
                    && let Some(import_path) = import_target_hints.get(&symbol)
                {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                        to_symbol_key: SymbolKey {
                            symbol: symbol.clone(),
                            qualified_symbol: Some(format!("{language}:{import_path}::{symbol}")),
                            file_path: Some(import_path.clone()),
                            language: Some(language.to_string()),
                        },
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                }
                if let Some(caller_symbol) = caller {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                        to_symbol_key: SymbolKey {
                            symbol,
                            qualified_symbol: None,
                            file_path: Some(file_path.to_string()),
                            language: Some(language.to_string()),
                        },
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                }
            }
        }
        "attribute" => {
            let object_symbol = node
                .child_by_field_name("object")
                .and_then(|object| node_text(object, source));
            let attribute_node = node.child_by_field_name("attribute").unwrap_or(node);
            let attribute_symbol = node_text(attribute_node, source).unwrap_or_default();
            if !attribute_symbol.is_empty() {
                let (line, column) = start_position(attribute_node);
                references.push(ExtractedReference {
                    symbol: attribute_symbol.clone(),
                    line,
                    column,
                });

                if let Some(caller_symbol) = caller
                    && let Some(object_symbol) = object_symbol
                    && let Some(import_path) = import_target_hints.get(&object_symbol)
                {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                        to_symbol_key: SymbolKey {
                            symbol: attribute_symbol.clone(),
                            qualified_symbol: Some(format!(
                                "{language}:{import_path}::{attribute_symbol}"
                            )),
                            file_path: Some(import_path.clone()),
                            language: Some(language.to_string()),
                        },
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                    return;
                }
            }
            collect_call_symbols(
                attribute_node,
                source,
                caller,
                file_path,
                language,
                import_target_hints,
                import_call_hints,
                references,
                edges,
            );
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_call_symbols(
                    child,
                    source,
                    caller,
                    file_path,
                    language,
                    import_target_hints,
                    import_call_hints,
                    references,
                    edges,
                );
            }
        }
    }
}

fn enclosing_class_name(node: Node<'_>, source: &str) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "class_definition"
            && let Some(name_node) = parent.child_by_field_name("name")
        {
            return node_text(name_node, source);
        }
        current = parent.parent();
    }
    None
}

fn enclosing_callable_name(node: Node<'_>, source: &str) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_definition"
            && let Some(name_node) = parent.child_by_field_name("name")
        {
            return node_text(name_node, source);
        }
        current = parent.parent();
    }
    None
}

fn module_constants(file_path: &str, language: &str, source: &str) -> Vec<ExtractedSymbol> {
    let mut symbols = Vec::new();

    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }

        let Some((left, _right)) = line.split_once('=') else {
            continue;
        };
        let candidate = left.trim();
        if candidate.is_empty() || !is_python_constant_name(candidate) {
            continue;
        }

        let start_column = line.find(candidate).unwrap_or(0) as u32 + 1;
        let line_no = index as u32 + 1;
        symbols.push(ExtractedSymbol {
            symbol: candidate.to_string(),
            qualified_symbol: Some(format!("{language}:{file_path}::{candidate}")),
            kind: "const".to_string(),
            language: language.to_string(),
            container: None,
            start_line: line_no,
            start_column,
            end_line: line_no,
            end_column: start_column + candidate.len() as u32,
            signature: Some(line.trim().to_string()),
        });
    }

    symbols
}

fn is_python_constant_name(candidate: &str) -> bool {
    let mut has_alpha = false;
    for ch in candidate.chars() {
        if !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
            return false;
        }
        if ch.is_ascii_uppercase() {
            has_alpha = true;
        }
    }

    has_alpha
}

#[derive(Debug)]
struct ImportBinding {
    local_symbol: String,
    imported_symbol: String,
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
}

fn import_bindings(node: Node<'_>, source: &str) -> Vec<ImportBinding> {
    let mut bindings = Vec::new();
    let statement = node_text(node, source).unwrap_or_default();
    let trimmed = statement.trim();
    let (start_line, start_column) = start_position(node);
    let (end_line, end_column) = end_position(node);

    if let Some(rest) = trimmed.strip_prefix("import ") {
        for specifier in rest.split(',') {
            let specifier = specifier.trim();
            if specifier.is_empty() {
                continue;
            }
            let (imported_path, local_alias) =
                if let Some((left, right)) = specifier.split_once(" as ") {
                    (left.trim(), Some(right.trim()))
                } else {
                    (specifier, None)
                };
            let Some(imported_symbol) = first_identifier(imported_path) else {
                continue;
            };
            let local_symbol = local_alias
                .map(str::to_string)
                .unwrap_or_else(|| imported_symbol.clone());
            bindings.push(ImportBinding {
                local_symbol,
                imported_symbol,
                start_line,
                start_column,
                end_line,
                end_column,
            });
        }
    }

    if let Some(rest) = trimmed.strip_prefix("from ")
        && let Some((_module, imports_part)) = rest.split_once(" import ")
    {
        for specifier in imports_part.split(',') {
            let specifier = specifier.trim();
            if specifier.is_empty() || specifier == "*" {
                continue;
            }
            let (imported_name, local_alias) =
                if let Some((left, right)) = specifier.split_once(" as ") {
                    (left.trim(), Some(right.trim()))
                } else {
                    (specifier, None)
                };
            let Some(imported_symbol) = last_identifier(imported_name) else {
                continue;
            };
            let local_symbol = local_alias
                .map(str::to_string)
                .unwrap_or_else(|| imported_symbol.clone());
            bindings.push(ImportBinding {
                local_symbol,
                imported_symbol,
                start_line,
                start_column,
                end_line,
                end_column,
            });
        }
    };

    bindings.sort_by(|left, right| {
        left.start_line
            .cmp(&right.start_line)
            .then(left.start_column.cmp(&right.start_column))
            .then(left.local_symbol.cmp(&right.local_symbol))
            .then(left.imported_symbol.cmp(&right.imported_symbol))
    });
    bindings.dedup_by(|left, right| {
        left.local_symbol == right.local_symbol && left.imported_symbol == right.imported_symbol
    });
    bindings
}

fn import_target_hints(file_path: &str, source: &str) -> HashMap<String, String> {
    let mut hints = HashMap::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("import ") {
            for specifier in rest.split(',') {
                let specifier = specifier.trim();
                if specifier.is_empty() {
                    continue;
                }
                let (imported_module, local_alias) =
                    if let Some((left, right)) = specifier.split_once(" as ") {
                        (left.trim(), Some(right.trim()))
                    } else {
                        (specifier, None)
                    };
                let Some(import_path) = resolve_python_import_path(file_path, imported_module)
                else {
                    continue;
                };
                let local_symbol = local_alias
                    .map(str::to_string)
                    .or_else(|| first_identifier(imported_module))
                    .unwrap_or_default();
                if local_symbol.is_empty() {
                    continue;
                }
                hints.insert(local_symbol, import_path);
            }
        } else if let Some(rest) = trimmed.strip_prefix("from ")
            && let Some((module_name, imports_part)) = rest.split_once(" import ")
        {
            for specifier in imports_part.split(',') {
                let specifier = specifier.trim();
                if specifier.is_empty() || specifier == "*" {
                    continue;
                }
                let (imported_name, local_alias) =
                    if let Some((left, right)) = specifier.split_once(" as ") {
                        (left.trim(), Some(right.trim()))
                    } else {
                        (specifier, None)
                    };
                let local_symbol = local_alias
                    .map(str::to_string)
                    .or_else(|| last_identifier(imported_name))
                    .unwrap_or_default();
                if local_symbol.is_empty() {
                    continue;
                }
                let import_path = resolve_python_import_path(
                    file_path,
                    &format!("{}.{}", module_name.trim(), imported_name.trim()),
                )
                .or_else(|| resolve_python_import_path(file_path, module_name.trim()));
                let Some(import_path) = import_path else {
                    continue;
                };
                hints.insert(local_symbol, import_path.clone());
            }
        }
    }

    hints
}

fn import_call_hints(file_path: &str, source: &str) -> HashMap<String, ImportCallHint> {
    let mut hints = HashMap::new();

    for line in source.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("from ") else {
            continue;
        };
        let Some((module_name, imports_part)) = rest.split_once(" import ") else {
            continue;
        };
        let Some(import_path) = resolve_python_import_path(file_path, module_name.trim()) else {
            continue;
        };

        for specifier in imports_part.split(',') {
            let specifier = specifier.trim();
            if specifier.is_empty() || specifier == "*" {
                continue;
            }
            let (imported_name, local_alias) =
                if let Some((left, right)) = specifier.split_once(" as ") {
                    (left.trim(), Some(right.trim()))
                } else {
                    (specifier, None)
                };
            let Some(imported_symbol) = last_identifier(imported_name) else {
                continue;
            };
            let local_symbol = local_alias
                .map(str::to_string)
                .unwrap_or_else(|| imported_symbol.clone());
            hints.insert(
                local_symbol,
                ImportCallHint {
                    import_path: import_path.clone(),
                    imported_symbol,
                },
            );
        }
    }

    hints
}

fn resolve_python_import_path(from_file_path: &str, module_name: &str) -> Option<String> {
    let module_path = module_name.trim();
    if module_path.is_empty() {
        return None;
    }

    if module_path.starts_with('.') {
        return resolve_relative_python_import_path(from_file_path, module_path);
    }

    let normalized_module_path = module_path.replace('.', "/");
    let mut components = Path::new(from_file_path).components();
    let first_component = components
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .filter(|component| !component.is_empty());

    if let Some(prefix) = first_component {
        Some(format!("{prefix}/{normalized_module_path}.py"))
    } else {
        Some(format!("{normalized_module_path}.py"))
    }
}

fn resolve_relative_python_import_path(from_file_path: &str, module_path: &str) -> Option<String> {
    let levels = module_path.chars().take_while(|ch| *ch == '.').count();
    if levels == 0 {
        return None;
    }

    let suffix = module_path[levels..].trim_matches('.');
    let mut resolved_path = Path::new(from_file_path).parent()?.to_path_buf();
    for _ in 1..levels {
        resolved_path = resolved_path.parent()?.to_path_buf();
    }

    if !suffix.is_empty() {
        for segment in suffix.split('.').filter(|segment| !segment.is_empty()) {
            resolved_path.push(segment);
        }
    }
    resolved_path.set_extension("py");
    Some(resolved_path.to_string_lossy().replace('\\', "/"))
}

fn last_identifier(text: &str) -> Option<String> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .rfind(|part| !part.is_empty())
        .map(str::to_string)
}

fn first_identifier(text: &str) -> Option<String> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|part| !part.is_empty())
        .map(str::to_string)
}

fn signature_summary(node: Node<'_>, source: &str) -> Option<String> {
    let text = node_text(node, source)?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_string())
}

fn node_text(node: Node<'_>, source: &str) -> Option<String> {
    source
        .as_bytes()
        .get(node.start_byte()..node.end_byte())
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .map(str::to_string)
}

fn start_position(node: Node<'_>) -> (u32, u32) {
    let position = node.start_position();
    (position.row as u32 + 1, position.column as u32 + 1)
}

fn end_position(node: Node<'_>) -> (u32, u32) {
    let position = node.end_position();
    (position.row as u32 + 1, position.column as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_python_root(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("python language should load");
        parser
            .parse(source, None)
            .expect("python source should parse")
    }

    fn find_nodes_of_kind<'a>(root: Node<'a>, kind: &str) -> Vec<Node<'a>> {
        let mut matches = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == kind {
                matches.push(node);
            }
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor).collect::<Vec<_>>();
            children.reverse();
            for child in children {
                stack.push(child);
            }
        }
        matches
    }

    #[test]
    fn resolve_python_import_path_supports_relative_module_paths() {
        let resolved = resolve_python_import_path("src/pkg/consumer.py", ".util");
        assert_eq!(resolved.as_deref(), Some("src/pkg/util.py"));
    }

    #[test]
    fn import_call_hints_capture_relative_from_import_paths() {
        let source = "from .util import helper\n\n\ndef run():\n    return helper()\n";
        let hints = import_call_hints("src/pkg/consumer.py", source);
        let hint = hints
            .get("helper")
            .expect("helper import should produce call hint");
        assert_eq!(hint.import_path, "src/pkg/util.py");
        assert_eq!(hint.imported_symbol, "helper");
    }

    #[test]
    fn import_helpers_cover_alias_and_star_paths() {
        let source = r#"
import pkg.tools as tools, pkg.other
from .local import run as local_run, *, helper
from pkg.mod import Build as MakeBuild
"#;

        let hints = import_target_hints("src/app/main.py", source);
        assert_eq!(
            hints.get("tools"),
            Some(&"src/pkg/tools.py".to_string()),
            "explicit import aliases should resolve"
        );
        assert_eq!(
            hints.get("pkg"),
            Some(&"src/pkg/other.py".to_string()),
            "non-aliased imports should use the left-most module token"
        );
        assert_eq!(
            hints.get("local_run"),
            Some(&"src/app/local/run.py".to_string()),
            "relative from-import aliases should resolve through file context"
        );
        assert!(
            !hints.contains_key("*"),
            "star imports should be skipped from import hints"
        );

        let call_hints = import_call_hints("src/app/main.py", source);
        let make_build = call_hints
            .get("MakeBuild")
            .expect("from-import alias should produce call hint");
        assert_eq!(make_build.import_path, "src/pkg/mod.py");
        assert_eq!(make_build.imported_symbol, "Build");
    }

    #[test]
    fn module_constant_and_name_validation_cover_rejections() {
        let constants = module_constants(
            "src/constants.py",
            "python",
            "GOOD_CONST = 1\nbadConst = 2\nINDENTED = 3\n    NESTED = 4\n",
        );
        assert!(
            constants.iter().any(|item| item.symbol == "GOOD_CONST"),
            "top-level constant should be extracted"
        );
        assert!(
            !constants.iter().any(|item| item.symbol == "badConst"),
            "mixed-case symbol should not be treated as module constant"
        );
        assert!(
            !constants.iter().any(|item| item.symbol == "NESTED"),
            "indented assignments should not be treated as module constants"
        );

        assert!(is_python_constant_name("HELLO_WORLD"));
        assert!(!is_python_constant_name("hello_world"));
        assert!(!is_python_constant_name("VALUE-1"));
    }

    #[test]
    fn resolve_python_import_path_covers_empty_relative_and_root_cases() {
        assert_eq!(resolve_python_import_path("src/app/main.py", ""), None);
        assert_eq!(
            resolve_python_import_path("src/app/main.py", "pkg.mod"),
            Some("src/pkg/mod.py".to_string())
        );
        assert_eq!(
            resolve_python_import_path("src/app/main.py", ".pkg.util"),
            Some("src/app/pkg/util.py".to_string())
        );
        assert_eq!(
            resolve_python_import_path("src/app/main.py", "..shared.util"),
            Some("src/shared/util.py".to_string())
        );
    }

    #[test]
    fn collect_call_symbols_covers_identifier_attribute_and_fallback_paths() {
        let source = r#"
import pkg.mod as mod
from pkg.factory import build as make_builder

def run():
    make_builder()
    mod.helper()
    object.method()
"#;
        let tree = parse_python_root(source);
        let root = tree.root_node();
        let call_nodes = find_nodes_of_kind(root, "call");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        let target_hints = import_target_hints("src/app/main.py", source);
        let call_hints = import_call_hints("src/app/main.py", source);
        for call in call_nodes {
            let function_node = call
                .child_by_field_name("function")
                .expect("call should have function child");
            collect_call_symbols(
                function_node,
                source,
                Some("run"),
                "src/app/main.py",
                "python",
                &target_hints,
                &call_hints,
                &mut references,
                &mut edges,
            );
        }

        assert!(
            references.iter().any(|item| item.symbol == "make_builder"),
            "identifier calls should be captured"
        );
        assert!(
            references.iter().any(|item| item.symbol == "helper"),
            "attribute calls should capture attribute symbol"
        );
        assert!(
            edges.iter().any(|edge| {
                edge.edge_kind == "calls"
                    && edge.to_symbol_key.qualified_symbol.as_deref()
                        == Some("python:src/pkg/factory.py::build")
            }),
            "from-import call hints should emit qualified call edges"
        );
        assert!(
            edges.iter().any(|edge| {
                edge.edge_kind == "calls"
                    && edge.to_symbol_key.qualified_symbol.as_deref()
                        == Some("python:src/pkg/mod.py::helper")
            }),
            "attribute import hints should emit module-qualified call edges"
        );
        assert!(
            edges.iter().any(|edge| {
                edge.edge_kind == "calls"
                    && edge.to_symbol_key.symbol == "method"
                    && edge.to_symbol_key.qualified_symbol.is_none()
            }),
            "non-import attribute calls should fall back to local symbol edges"
        );
    }

    #[test]
    fn adapter_extract_covers_imports_constants_and_contains_edges() {
        let source = r#"
import pkg.mod as mod
from pkg.factory import build as make_builder

class Worker:
    def run(self):
        make_builder()
        mod.helper()

def helper():
    return 1

TOP_LEVEL = 7
"#;

        let unit = PythonLanguageAdapter
            .extract("src/app/main.py", source)
            .expect("python extraction should succeed");
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "class" && item.symbol == "Worker"),
            "class definitions should be emitted"
        );
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "method" && item.symbol == "run"),
            "method definitions should be emitted"
        );
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "const" && item.symbol == "TOP_LEVEL"),
            "module constants should be emitted"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "contains"
                    && edge.from_symbol_key.symbol == "Worker"
                    && edge.to_symbol_key.symbol == "run"
            }),
            "class/method nesting should emit contains edges"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "imports" && edge.from_symbol_key.symbol == "mod"
            }),
            "import definitions should emit import edges"
        );
    }

    #[test]
    fn helper_paths_cover_dedup_sort_fallback_and_import_guard_branches() {
        let source = r#"
import pkg.mod as mod
import pkg.mod as mod

def dupe():
    pass

def dupe():
    pass

def run():
    mod.call()
    mod.call()
"#;
        let unit = PythonLanguageAdapter
            .extract("src/app/main.py", source)
            .expect("python extraction should succeed");
        assert!(
            unit.symbols.iter().any(|item| item.symbol == "dupe"),
            "duplicate symbol extraction should remain deterministic"
        );

        let tree = parse_python_root(source);
        let root = tree.root_node();
        let import_node = find_nodes_of_kind(root, "import_statement")
            .into_iter()
            .next()
            .expect("import statement should exist");
        let mut defs = Vec::new();
        assert_eq!(
            push_named_definition(
                import_node,
                source,
                "function",
                None,
                "src/app/main.py",
                "python",
                &mut defs
            ),
            None,
            "non-identifier name nodes should short-circuit"
        );

        let mod_identifier = find_nodes_of_kind(root, "identifier")
            .into_iter()
            .find(|node| node_text(*node, source).as_deref() == Some("mod"))
            .expect("module alias identifier should exist");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        collect_call_symbols(
            mod_identifier,
            source,
            Some("run"),
            "src/app/main.py",
            "python",
            &import_target_hints("src/app/main.py", source),
            &HashMap::new(),
            &mut references,
            &mut edges,
        );
        assert!(
            !edges.is_empty(),
            "identifier calls should use import-target hints when call hints are absent"
        );

        let mut recursive_refs = Vec::new();
        let mut recursive_edges = Vec::new();
        collect_call_symbols(
            root,
            source,
            Some("run"),
            "src/app/main.py",
            "python",
            &import_target_hints("src/app/main.py", source),
            &import_call_hints("src/app/main.py", source),
            &mut recursive_refs,
            &mut recursive_edges,
        );
        assert!(
            !recursive_refs.is_empty(),
            "fallback recursion should traverse non-call root nodes"
        );
        assert_eq!(enclosing_callable_name(root, source), None);

        let malformed_imports = "import pkg.tools as tools, , pkg.other as \n";
        let malformed_tree = parse_python_root(malformed_imports);
        let malformed_root = malformed_tree.root_node();
        let malformed_bindings = import_bindings(malformed_root, malformed_imports);
        assert!(
            malformed_bindings
                .iter()
                .any(|binding| binding.local_symbol == "tools"),
            "valid import aliases should remain after malformed specifiers are skipped"
        );

        let from_imports = "from pkg.mod import helper as local, *, ... as bad, value as ";
        let from_tree = parse_python_root(from_imports);
        let from_root = from_tree.root_node();
        let from_node = find_nodes_of_kind(from_root, "import_from_statement")
            .into_iter()
            .next()
            .expect("from import statement should parse");
        let from_bindings = import_bindings(from_node, from_imports);
        assert!(
            from_bindings
                .iter()
                .any(|binding| binding.local_symbol == "local"),
            "from-import aliases should be preserved"
        );

        let hint_source = "\
import pkg.tools as tools, , unknown as \n\
from pkg.mod import helper as local, *, ... as bad, value as \n\
from missingline\n";
        let hints = import_target_hints("src/app/main.py", hint_source);
        assert_eq!(
            hints.get("tools"),
            Some(&"src/pkg/tools.py".to_string()),
            "valid import aliases should resolve to module paths"
        );

        let call_hint_source = "\
from pkg.mod import helper as local, value as \n\
from  import missing\n\
from missingline\n";
        let call_hints = import_call_hints("src/app/main.py", call_hint_source);
        assert!(
            call_hints.contains_key("local"),
            "valid from-import call hints should remain after malformed lines are skipped"
        );

        assert_eq!(
            resolve_python_import_path("", "pkg.mod"),
            Some("pkg/mod.py".to_string())
        );
        assert_eq!(
            resolve_relative_python_import_path("src/app/main.py", "pkg.mod"),
            None
        );

        let blank_tree = parse_python_root("\n");
        assert_eq!(
            signature_summary(blank_tree.root_node(), "\n"),
            None,
            "empty signature lines should be rejected"
        );
    }

    #[test]
    fn helper_paths_cover_remaining_malformed_import_and_attribute_fallbacks() {
        let duplicate_import_source = "import pkg.mod as mod, pkg.mod as mod\n";
        let duplicate_unit = PythonLanguageAdapter
            .extract("src/app/main.py", duplicate_import_source)
            .expect("python extraction should succeed");
        assert_eq!(
            duplicate_unit
                .symbols
                .iter()
                .filter(|item| item.kind == "import" && item.symbol == "mod")
                .count(),
            1,
            "duplicate same-line import bindings should deduplicate by symbol position"
        );

        let attribute_source = "def run():\n    object.method()\n";
        let attribute_tree = parse_python_root(attribute_source);
        let call_node = find_nodes_of_kind(attribute_tree.root_node(), "call")
            .into_iter()
            .next()
            .expect("attribute call should parse");
        let attribute_node = call_node
            .child_by_field_name("function")
            .expect("call should expose function field");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        collect_call_symbols(
            attribute_node,
            attribute_source,
            Some("run"),
            "src/app/main.py",
            "python",
            &HashMap::new(),
            &HashMap::new(),
            &mut references,
            &mut edges,
        );
        assert!(
            references.iter().any(|item| item.symbol == "method"),
            "attribute call helper should record referenced method symbols"
        );

        let import_line_source = "import ... as alias\n";
        let import_line_tree = parse_python_root(import_line_source);
        assert!(
            import_bindings(import_line_tree.root_node(), import_line_source).is_empty(),
            "invalid import identifiers should be skipped"
        );

        let from_line_source = "from pkg import ... as alias\n";
        let from_line_tree = parse_python_root(from_line_source);
        assert!(
            import_bindings(from_line_tree.root_node(), from_line_source).is_empty(),
            "invalid from-import identifiers should be skipped"
        );

        let hints = import_target_hints(
            "src/app/main.py",
            "\
import .... as alias\n\
import pkg.mod as \n\
from pkg.mod import value as \n\
from ....pkg import value\n",
        );
        assert!(
            !hints.contains_key(""),
            "import hints should ignore malformed aliases that normalize to empty symbols"
        );

        let call_hints = import_call_hints(
            "src/app/main.py",
            "from pkg.mod import ... as alias, value as \n",
        );
        assert!(
            !call_hints.contains_key(""),
            "call hints should ignore malformed aliases that normalize to empty symbols"
        );

        let malformed_only_hints =
            import_target_hints("src/app/main.py", "import ...\nfrom pkg.mod import ...\n");
        assert!(
            malformed_only_hints.is_empty(),
            "malformed import symbols should be ignored when local names are empty"
        );
        let malformed_only_call_hints =
            import_call_hints("src/app/main.py", "from pkg.mod import ...\n");
        assert!(
            malformed_only_call_hints.is_empty(),
            "malformed from-import call hints should be skipped when local symbols are empty"
        );

        assert_eq!(
            resolve_relative_python_import_path("src/app/main.py", ".pkg.util"),
            Some("src/app/pkg/util.py".to_string())
        );

        let blank_tree = parse_python_root("   \n");
        assert_eq!(signature_summary(blank_tree.root_node(), "   \n"), None);
    }

    #[test]
    fn collect_call_symbols_covers_identifier_and_attribute_fallback_closing_paths() {
        let source = r#"
def run():
    helper()
    obj.render()
"#;
        let tree = parse_python_root(source);
        let root = tree.root_node();
        let call_nodes = find_nodes_of_kind(root, "call");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        for call in call_nodes {
            let function_node = call
                .child_by_field_name("function")
                .expect("call should expose function child");
            collect_call_symbols(
                function_node,
                source,
                Some("run"),
                "src/app/main.py",
                "python",
                &HashMap::new(),
                &HashMap::new(),
                &mut references,
                &mut edges,
            );
        }
        assert!(
            references.iter().any(|item| item.symbol == "helper"),
            "identifier call symbols should be captured"
        );
        assert!(
            references.iter().any(|item| item.symbol == "render"),
            "attribute call symbols should be captured"
        );
        assert!(
            edges
                .iter()
                .any(|edge| edge.to_symbol_key.symbol == "helper"),
            "identifier fallback should emit local call edges"
        );
        assert!(
            edges
                .iter()
                .any(|edge| edge.to_symbol_key.symbol == "render"),
            "attribute fallback should emit local call edges"
        );
    }

    #[test]
    fn collect_call_symbols_covers_empty_identifier_and_attribute_symbol_paths() {
        let source = r#"
def run():
    helper()
    obj.render()
"#;
        let tree = parse_python_root(source);
        let root = tree.root_node();
        let call_nodes = find_nodes_of_kind(root, "call");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        for call in call_nodes {
            let function_node = call
                .child_by_field_name("function")
                .expect("call should expose function child");
            collect_call_symbols(
                function_node,
                "",
                Some("run"),
                "src/app/main.py",
                "python",
                &HashMap::new(),
                &HashMap::new(),
                &mut references,
                &mut edges,
            );
        }
        assert!(
            references.is_empty() && edges.is_empty(),
            "mismatched source bytes should short-circuit empty symbol paths without panicking"
        );
    }

    #[test]
    fn signature_summary_returns_first_nonempty_definition_line() {
        let source = "\ndef run():\n    return 1\n";
        let tree = parse_python_root(source);
        let root = tree.root_node();
        assert_eq!(
            signature_summary(root, source),
            Some("def run():".to_string())
        );
    }
}
