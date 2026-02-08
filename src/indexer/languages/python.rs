use anyhow::Context;
use std::collections::HashMap;
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
            left.symbol == right.symbol
                && left.kind == right.kind
                && left.start_line == right.start_line
                && left.start_column == right.start_column
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
            if let Some(symbol) = node_text(node, source) {
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
            if let Some(attribute_node) = node.child_by_field_name("attribute") {
                if let Some(attribute_symbol) = node_text(attribute_node, source) {
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
            } else if let Some(text) = node_text(node, source)
                && let Some(symbol) = last_identifier(&text)
            {
                let (line, column) = start_position(node);
                references.push(ExtractedReference {
                    symbol: symbol.clone(),
                    line,
                    column,
                });
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
    } else if let Some(rest) = trimmed.strip_prefix("from ")
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
    }

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
            if local_symbol.is_empty() {
                continue;
            }
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
    if module_path.is_empty() || module_path.starts_with('.') {
        return None;
    }

    let normalized_module_path = module_path.replace('.', "/");
    let mut components = std::path::Path::new(from_file_path).components();
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
    if line.is_empty() {
        return None;
    }
    Some(line.to_string())
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
