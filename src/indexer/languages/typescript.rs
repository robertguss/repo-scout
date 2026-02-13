use anyhow::Context;
use std::collections::HashMap;
use tree_sitter::{Node, Parser};

use crate::indexer::languages::{
    ExtractedEdge, ExtractedReference, ExtractedSymbol, ExtractionUnit, LanguageAdapter, SymbolKey,
};

pub struct TypeScriptLanguageAdapter;

#[derive(Debug, Clone)]
struct ImportCallHint {
    import_paths: Vec<String>,
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

impl LanguageAdapter for TypeScriptLanguageAdapter {
    fn language_id(&self) -> &'static str {
        "typescript"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx"]
    }

    fn extract(&self, file_path: &str, source: &str) -> anyhow::Result<ExtractionUnit> {
        let mut parser = Parser::new();
        if file_path.ends_with(".tsx") {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
                .context("failed to load tree-sitter tsx language")?;
        } else {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .context("failed to load tree-sitter typescript language")?;
        }

        let tree = parser
            .parse(source, None)
            .context("failed to parse typescript source")?;

        let language = self.language_id().to_string();
        let import_target_hints = import_target_hints(file_path, source);
        let import_call_hints = import_call_hints(file_path, source);
        let mut symbols = Vec::new();
        let mut references = Vec::new();
        let mut edges = Vec::new();
        let mut stack = vec![tree.root_node()];

        while let Some(node) = stack.pop() {
            match node.kind() {
                "function_declaration" => {
                    push_named_definition(
                        node,
                        source,
                        "function",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "class_declaration" => {
                    let class_name = push_named_definition(
                        node,
                        source,
                        "class",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );

                    if let Some(class_symbol) = class_name {
                        for implemented in implemented_types(node, source) {
                            if let Some(import_paths) = import_target_hints.get(&implemented) {
                                for import_path in import_paths {
                                    edges.push(ExtractedEdge {
                                        from_symbol_key: scoped_symbol_key(
                                            file_path,
                                            &language,
                                            &class_symbol,
                                        ),
                                        to_symbol_key: SymbolKey {
                                            symbol: implemented.clone(),
                                            qualified_symbol: Some(format!(
                                                "{language}:{import_path}::{implemented}"
                                            )),
                                            file_path: Some(import_path.clone()),
                                            language: Some(language.clone()),
                                        },
                                        edge_kind: "implements".to_string(),
                                        confidence: 0.95,
                                        provenance: "ast_reference".to_string(),
                                    });
                                }
                            } else {
                                edges.push(ExtractedEdge {
                                    from_symbol_key: scoped_symbol_key(
                                        file_path,
                                        &language,
                                        &class_symbol,
                                    ),
                                    to_symbol_key: language_symbol_key(&implemented, &language),
                                    edge_kind: "implements".to_string(),
                                    confidence: 0.95,
                                    provenance: "ast_reference".to_string(),
                                });
                            }
                        }
                    }
                }
                "interface_declaration" => {
                    push_named_definition(
                        node,
                        source,
                        "interface",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "enum_declaration" => {
                    push_named_definition(
                        node,
                        source,
                        "enum",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "type_alias_declaration" => {
                    push_named_definition(
                        node,
                        source,
                        "type_alias",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "variable_declarator" => {
                    let is_callable_value = node
                        .child_by_field_name("value")
                        .is_some_and(|value| matches!(value.kind(), "arrow_function" | "function"));
                    if is_callable_value {
                        push_named_definition(
                            node,
                            source,
                            "variable",
                            None,
                            file_path,
                            &language,
                            &mut symbols,
                        );
                    }
                }
                "method_definition" => {
                    let container = enclosing_class_name(node, source);
                    if let Some(symbol) = push_named_definition(
                        node,
                        source,
                        "method",
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
                "call_expression" => {
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
                "import_statement" => {
                    for binding in import_bindings(node, source) {
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
                        if let Some(import_paths) = import_target_hints.get(&binding.local_symbol) {
                            for import_path in import_paths {
                                edges.push(ExtractedEdge {
                                    from_symbol_key: scoped_symbol_key(
                                        file_path,
                                        &language,
                                        &binding.local_symbol,
                                    ),
                                    to_symbol_key: SymbolKey {
                                        symbol: binding.imported_symbol.clone(),
                                        qualified_symbol: Some(format!(
                                            "{language}:{import_path}::{}",
                                            binding.imported_symbol
                                        )),
                                        file_path: Some(import_path.clone()),
                                        language: Some(language.clone()),
                                    },
                                    edge_kind: "imports".to_string(),
                                    confidence: 0.9,
                                    provenance: "import_resolution".to_string(),
                                });
                            }
                        } else {
                            edges.push(ExtractedEdge {
                                from_symbol_key: scoped_symbol_key(
                                    file_path,
                                    &language,
                                    &binding.local_symbol,
                                ),
                                to_symbol_key: language_symbol_key(
                                    &binding.imported_symbol,
                                    &language,
                                ),
                                edge_kind: "imports".to_string(),
                                confidence: 0.9,
                                provenance: "import_resolution".to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        symbols.sort_by(|left, right| {
            left.start_line
                .cmp(&right.start_line)
                .then(left.start_column.cmp(&right.start_column))
                .then(left.symbol.cmp(&right.symbol))
                .then(left.kind.cmp(&right.kind))
        });
        references.sort_by(|left, right| {
            left.line
                .cmp(&right.line)
                .then(left.column.cmp(&right.column))
                .then(left.symbol.cmp(&right.symbol))
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
    if !matches!(
        name_node.kind(),
        "identifier" | "type_identifier" | "property_identifier"
    ) {
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

fn implemented_types(node: Node<'_>, source: &str) -> Vec<String> {
    let mut implemented = Vec::new();
    if let Some(text) = node_text(node, source)
        && let Some(index) = text.find("implements")
    {
        let after = &text[index + "implements".len()..];
        let clause = after.split('{').next().unwrap_or(after);
        for part in clause.split(',') {
            let candidate = part
                .trim()
                .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .find(|token| !token.is_empty());
            if let Some(symbol) = candidate {
                implemented.push(symbol.to_string());
            }
        }
    }

    if implemented.is_empty() {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "implements_clause" | "class_heritage") {
                collect_type_identifiers(child, source, &mut implemented);
            }
        }
    }

    implemented.sort();
    implemented.dedup();
    implemented
}

fn collect_type_identifiers(node: Node<'_>, source: &str, output: &mut Vec<String>) {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if matches!(current.kind(), "identifier" | "type_identifier")
            && let Some(symbol) = node_text(current, source)
        {
            output.push(symbol);
            continue;
        }
        let mut cursor = current.walk();
        let mut children = current.children(&mut cursor).collect::<Vec<_>>();
        children.reverse();
        for child in children {
            stack.push(child);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_call_symbols(
    node: Node<'_>,
    source: &str,
    caller: Option<&str>,
    file_path: &str,
    language: &str,
    import_target_hints: &HashMap<String, Vec<String>>,
    import_call_hints: &HashMap<String, ImportCallHint>,
    references: &mut Vec<ExtractedReference>,
    edges: &mut Vec<ExtractedEdge>,
) {
    match node.kind() {
        "identifier" | "property_identifier" => {
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
                    for import_path in &call_hint.import_paths {
                        edges.push(ExtractedEdge {
                            from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                            to_symbol_key: SymbolKey {
                                symbol: call_hint.imported_symbol.clone(),
                                qualified_symbol: Some(format!(
                                    "{language}:{import_path}::{}",
                                    call_hint.imported_symbol
                                )),
                                file_path: Some(import_path.clone()),
                                language: Some(language.to_string()),
                            },
                            edge_kind: "calls".to_string(),
                            confidence: 0.95,
                            provenance: "call_resolution".to_string(),
                        });
                    }
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
        "member_expression" => {
            let object_symbol = node
                .child_by_field_name("object")
                .and_then(|object| node_text(object, source));
            let property = node.child_by_field_name("property").unwrap_or(node);
            let property_symbol = node_text(property, source).unwrap_or_default();
            if !property_symbol.is_empty() {
                let (line, column) = start_position(property);
                references.push(ExtractedReference {
                    symbol: property_symbol.clone(),
                    line,
                    column,
                });

                if let Some(caller_symbol) = caller
                    && let Some(object_symbol) = object_symbol
                    && let Some(import_paths) = import_target_hints.get(&object_symbol)
                {
                    for import_path in import_paths {
                        edges.push(ExtractedEdge {
                            from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                            to_symbol_key: SymbolKey {
                                symbol: property_symbol.clone(),
                                qualified_symbol: Some(format!(
                                    "{language}:{import_path}::{property_symbol}"
                                )),
                                file_path: Some(import_path.clone()),
                                language: Some(language.to_string()),
                            },
                            edge_kind: "calls".to_string(),
                            confidence: 0.95,
                            provenance: "call_resolution".to_string(),
                        });
                    }
                    return;
                }
            }
            collect_call_symbols(
                property,
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
        if parent.kind() == "class_declaration"
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
        if matches!(parent.kind(), "function_declaration" | "method_definition")
            && let Some(name_node) = parent.child_by_field_name("name")
        {
            return node_text(name_node, source);
        }
        if matches!(parent.kind(), "arrow_function" | "function")
            && let Some(declarator) = parent.parent()
            && declarator.kind() == "variable_declarator"
            && let Some(name_node) = declarator.child_by_field_name("name")
        {
            return node_text(name_node, source);
        }
        current = parent.parent();
    }
    None
}

fn signature_summary(node: Node<'_>, source: &str) -> Option<String> {
    let text = node_text(node, source)?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_string())
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
    let (start_line, start_column) = start_position(node);
    let (end_line, end_column) = end_position(node);

    if let (Some(left_brace), Some(right_rel)) = (
        statement.find('{'),
        statement
            .get(statement.find('{').unwrap_or(0) + 1..)
            .and_then(|rest| rest.find('}')),
    ) {
        let right_brace = left_brace + 1 + right_rel;
        let clause = &statement[left_brace + 1..right_brace];
        for specifier in clause.split(',') {
            let specifier = specifier.trim();
            if specifier.is_empty() {
                continue;
            }
            let (imported_symbol, local_symbol) =
                if let Some((imported, local)) = specifier.split_once(" as ") {
                    (imported.trim(), local.trim())
                } else {
                    (specifier, specifier)
                };
            bindings.push(ImportBinding {
                local_symbol: local_symbol.to_string(),
                imported_symbol: imported_symbol.to_string(),
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

fn import_target_hints(file_path: &str, source: &str) -> HashMap<String, Vec<String>> {
    let mut hints = HashMap::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") {
            continue;
        }
        let Some((head, from_tail)) = trimmed.split_once(" from ") else {
            continue;
        };
        let Some(module_specifier) = quoted_text(from_tail) else {
            continue;
        };
        let import_paths = resolve_typescript_import_paths(file_path, &module_specifier);
        if import_paths.is_empty() {
            continue;
        }
        let clause = head.trim_start_matches("import").trim();

        if let Some(namespace_clause) = clause
            .split(',')
            .find(|part| part.trim_start().starts_with("* as "))
        {
            let alias = namespace_clause.trim().trim_start_matches("* as ").trim();
            if !alias.is_empty() {
                extend_import_hint(&mut hints, alias, &import_paths);
            }

            let default_binding = clause
                .split(',')
                .next()
                .map(str::trim)
                .filter(|candidate| !candidate.is_empty() && !candidate.starts_with("* as "));
            if let Some(default_binding) = default_binding {
                extend_import_hint(&mut hints, default_binding, &import_paths);
            }
            continue;
        }

        if let (Some(left_brace), Some(right_brace)) = (head.find('{'), head.find('}')) {
            let clause = &head[left_brace + 1..right_brace];
            for specifier in clause.split(',') {
                let specifier = specifier.trim();
                if specifier.is_empty() {
                    continue;
                }
                let local_symbol = if let Some((_imported, local)) = specifier.split_once(" as ") {
                    local.trim()
                } else {
                    specifier
                };
                extend_import_hint(&mut hints, local_symbol, &import_paths);
            }
            let default_binding = head[..left_brace]
                .trim_start_matches("import")
                .trim()
                .trim_end_matches(',')
                .trim();
            if !default_binding.is_empty() {
                extend_import_hint(&mut hints, default_binding, &import_paths);
            }
            continue;
        }

        let default_binding = head
            .trim_start_matches("import")
            .trim()
            .split(',')
            .next()
            .map(str::trim)
            .filter(|symbol| !symbol.is_empty());
        if let Some(local_symbol) = default_binding {
            extend_import_hint(&mut hints, local_symbol, &import_paths);
        }
    }

    hints
}

fn extend_import_hint(
    hints: &mut HashMap<String, Vec<String>>,
    local_symbol: &str,
    import_paths: &[String],
) {
    let entry = hints.entry(local_symbol.to_string()).or_default();
    entry.extend(import_paths.iter().cloned());
    entry.sort();
    entry.dedup();
}

fn import_call_hints(file_path: &str, source: &str) -> HashMap<String, ImportCallHint> {
    let mut hints = HashMap::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") {
            continue;
        }
        let Some((head, from_tail)) = trimmed.split_once(" from ") else {
            continue;
        };
        let Some(module_specifier) = quoted_text(from_tail) else {
            continue;
        };
        let import_paths = resolve_typescript_import_paths(file_path, &module_specifier);
        if import_paths.is_empty() {
            continue;
        }
        let Some((left_brace, right_brace)) = head
            .find('{')
            .and_then(|left| head.find('}').map(|right| (left, right)))
        else {
            continue;
        };
        if right_brace <= left_brace {
            continue;
        }

        let clause = &head[left_brace + 1..right_brace];
        for specifier in clause.split(',') {
            let specifier = specifier.trim();
            if specifier.is_empty() {
                continue;
            }
            let (imported_symbol, local_symbol) =
                if let Some((imported, local)) = specifier.split_once(" as ") {
                    (imported.trim(), local.trim())
                } else {
                    (specifier, specifier)
                };
            hints.insert(
                local_symbol.to_string(),
                ImportCallHint {
                    import_paths: import_paths.clone(),
                    imported_symbol: imported_symbol.to_string(),
                },
            );
        }
    }

    hints
}

fn quoted_text(segment: &str) -> Option<String> {
    let start = segment.find(['"', '\''])?;
    let quote = segment.as_bytes()[start] as char;
    let tail = &segment[start + 1..];
    let end = tail.find(quote)?;
    Some(tail[..end].to_string())
}

fn resolve_typescript_import_paths(from_file_path: &str, module_specifier: &str) -> Vec<String> {
    if !module_specifier.starts_with('.') {
        return Vec::new();
    }

    let Some(base) = std::path::Path::new(from_file_path).parent() else {
        return Vec::new();
    };
    let candidate = base.join(module_specifier);
    let mut resolved = Vec::new();

    if candidate.extension().is_some() {
        resolved.push(normalize_relative_path(&candidate));
    } else {
        for extension in ["ts", "tsx"] {
            let mut direct = candidate.clone();
            direct.set_extension(extension);
            resolved.push(normalize_relative_path(&direct));
            resolved.push(normalize_relative_path(
                &candidate.join(format!("index.{extension}")),
            ));
        }
    }

    resolved.sort();
    resolved.dedup();
    resolved
}

fn normalize_relative_path(path: &std::path::Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    parts.join("/")
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

    fn parse_typescript_root(source: &str, tsx: bool) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        if tsx {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
                .expect("tsx language should load");
        } else {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .expect("typescript language should load");
        }
        parser
            .parse(source, None)
            .expect("typescript source should parse")
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
    fn import_hint_helpers_cover_named_default_namespace_and_call_hints() {
        let source = r#"
import DefaultApi, { Build as MakeBuild, util } from "./pkg/api";
import * as NS from "./pkg/ns";
import sideEffect from "./pkg/side";
"#;
        let hints = import_target_hints("src/app/main.ts", source);
        let expected_paths = vec![
            "src/app/pkg/api.ts".to_string(),
            "src/app/pkg/api.tsx".to_string(),
            "src/app/pkg/api/index.ts".to_string(),
            "src/app/pkg/api/index.tsx".to_string(),
        ];
        assert_eq!(hints.get("DefaultApi"), Some(&expected_paths));
        assert_eq!(hints.get("MakeBuild"), Some(&expected_paths));
        assert_eq!(hints.get("util"), Some(&expected_paths));
        assert_eq!(
            hints.get("NS"),
            Some(&vec![
                "src/app/pkg/ns.ts".to_string(),
                "src/app/pkg/ns.tsx".to_string(),
                "src/app/pkg/ns/index.ts".to_string(),
                "src/app/pkg/ns/index.tsx".to_string(),
            ])
        );
        assert_eq!(
            hints.get("sideEffect"),
            Some(&vec![
                "src/app/pkg/side.ts".to_string(),
                "src/app/pkg/side.tsx".to_string(),
                "src/app/pkg/side/index.ts".to_string(),
                "src/app/pkg/side/index.tsx".to_string(),
            ])
        );

        let call_hints = import_call_hints("src/app/main.ts", source);
        let make_build = call_hints
            .get("MakeBuild")
            .expect("aliased named import should create call hint");
        assert_eq!(make_build.imported_symbol, "Build");
        assert_eq!(make_build.import_paths, expected_paths);
    }

    #[test]
    fn resolve_path_and_quote_helpers_cover_relative_and_non_relative_cases() {
        assert_eq!(quoted_text("from \"./x\""), Some("./x".to_string()));
        assert_eq!(quoted_text("from './x'"), Some("./x".to_string()));
        assert_eq!(quoted_text("from ./x"), None);

        assert!(resolve_typescript_import_paths("src/main.ts", "react").is_empty());
        assert_eq!(
            resolve_typescript_import_paths("src/main.ts", "./direct.ts"),
            vec!["src/direct.ts".to_string()]
        );
        assert_eq!(
            resolve_typescript_import_paths("src/main.ts", "./mod"),
            vec![
                "src/mod.ts".to_string(),
                "src/mod.tsx".to_string(),
                "src/mod/index.ts".to_string(),
                "src/mod/index.tsx".to_string(),
            ]
        );
        assert_eq!(
            normalize_relative_path(std::path::Path::new("src/app/../pkg/./view.ts")),
            "src/pkg/view.ts".to_string()
        );
    }

    #[test]
    fn adapter_extract_covers_tsx_parser_implements_and_call_edges() {
        let source = r#"
import DefaultApi, { Build as MakeBuild } from "./api";
import { External } from "./types";
import * as NS from "./ns";

interface Shape {}

class Worker implements Shape, External {
  run() {
    MakeBuild();
    NS.render();
  }
}

const helper = () => <div>ok</div>;
"#;
        let unit = TypeScriptLanguageAdapter
            .extract("src/app/view.tsx", source)
            .expect("tsx extraction should succeed");

        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "class" && item.symbol == "Worker"),
            "class declarations should be extracted"
        );
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "method" && item.symbol == "run"),
            "method definitions should be extracted"
        );
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "variable" && item.symbol == "helper"),
            "callable variable declarations should be extracted"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "contains"
                    && edge.from_symbol_key.symbol == "Worker"
                    && edge.to_symbol_key.symbol == "run"
            }),
            "class methods should emit contains edges"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "implements"
                    && edge.to_symbol_key.symbol == "Shape"
                    && edge.to_symbol_key.qualified_symbol.is_none()
            }),
            "non-import implemented types should fall back to language-wide key"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "implements"
                    && edge.to_symbol_key.symbol == "External"
                    && edge
                        .to_symbol_key
                        .qualified_symbol
                        .as_deref()
                        .is_some_and(|qualified| qualified.contains("src/app/types"))
            }),
            "imported implemented types should emit qualified edges"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "calls"
                    && edge.to_symbol_key.symbol == "Build"
                    && edge
                        .to_symbol_key
                        .qualified_symbol
                        .as_deref()
                        .is_some_and(|qualified| qualified.contains("src/app/api"))
            }),
            "import call hints should drive call edges for named imports"
        );
    }

    #[test]
    fn helper_functions_cover_fallback_branches() {
        let source = r#"
class Child extends Base {}
const [first] = [1];
const fnVar = () => {
  runTask();
};
"#;
        let tree = parse_typescript_root(source, false);
        let root = tree.root_node();

        let class_node = find_nodes_of_kind(root, "class_declaration")
            .into_iter()
            .next()
            .expect("class declaration should exist");
        assert_eq!(
            implemented_types(class_node, source),
            vec!["Base".to_string()]
        );

        let declarators = find_nodes_of_kind(root, "variable_declarator");
        let array_binding_declarator = declarators
            .iter()
            .copied()
            .find(|node| {
                node.child_by_field_name("name")
                    .is_some_and(|name| name.kind() == "array_pattern")
            })
            .expect("array binding declarator should exist");
        let mut output = Vec::new();
        assert_eq!(
            push_named_definition(
                array_binding_declarator,
                source,
                "variable",
                None,
                "src/app/main.ts",
                "typescript",
                &mut output
            ),
            None
        );
        assert!(output.is_empty());

        let call_expression = find_nodes_of_kind(root, "call_expression")
            .into_iter()
            .next()
            .expect("call expression should exist");
        assert_eq!(
            enclosing_callable_name(call_expression, source),
            Some("fnVar".to_string())
        );

        let import_tree = parse_typescript_root("import { a as b, , c } from \"./x\";", false);
        let import_nodes = find_nodes_of_kind(import_tree.root_node(), "import_statement");
        let bindings = import_bindings(
            *import_nodes.first().expect("import statement should exist"),
            "import { a as b, , c } from \"./x\";",
        );
        assert_eq!(
            bindings
                .iter()
                .map(|binding| (
                    binding.imported_symbol.clone(),
                    binding.local_symbol.clone()
                ))
                .collect::<Vec<_>>(),
            vec![
                ("a".to_string(), "b".to_string()),
                ("c".to_string(), "c".to_string())
            ]
        );
    }

    #[test]
    fn helper_paths_cover_import_fallbacks_and_recursive_call_branches() {
        let source = r#"
import { Build as BuildAlias, Build as BuildAlias } from "react";
import DefaultApi, * as NS from "./pkg/ns";

class Worker implements External {
  run() {
    BuildAlias();
    NS.render();
  }
}

const helper = () => {
  BuildAlias();
};
"#;
        let unit = TypeScriptLanguageAdapter
            .extract("src/app/main.ts", source)
            .expect("typescript extraction should succeed");
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "imports" && edge.from_symbol_key.symbol == "BuildAlias"
            }),
            "non-relative imports should still emit fallback language-level import edges"
        );

        let tree = parse_typescript_root(source, false);
        let root = tree.root_node();
        assert_eq!(enclosing_class_name(root, source), None);

        let mut references = Vec::new();
        let mut edges = Vec::new();
        collect_call_symbols(
            root,
            source,
            Some("run"),
            "src/app/main.ts",
            "typescript",
            &import_target_hints("src/app/main.ts", source),
            &import_call_hints("src/app/main.ts", source),
            &mut references,
            &mut edges,
        );
        assert!(
            !references.is_empty(),
            "fallback recursion should walk child nodes and gather call references"
        );

        let malformed_import = "import { a as b, a as b, c as  } from \"./x\";";
        let malformed_tree = parse_typescript_root(malformed_import, false);
        let malformed_statement =
            find_nodes_of_kind(malformed_tree.root_node(), "import_statement")
                .into_iter()
                .next()
                .expect("import statement should parse");
        let bindings = import_bindings(malformed_statement, malformed_import);
        assert!(
            bindings
                .iter()
                .any(|binding| binding.local_symbol == "b" && binding.imported_symbol == "a"),
            "valid named import aliases should be retained"
        );

        let hints_source = "\
import { a as alias, b as  } from \"./pkg/mod\";\n\
import { c } from react;\n\
import MissingFromClause;\n";
        let hints = import_target_hints("src/app/main.ts", hints_source);
        assert!(
            hints.contains_key("alias"),
            "relative named imports should still be captured in hints"
        );
        assert!(
            !hints.contains_key("c"),
            "non-relative imports should be ignored for file-path hints"
        );

        let call_hints_source = "\
import { build as makeBuild, c as  } from \"./pkg/mod\";\n\
import { bad } from react;\n\
import MissingFromClause;\n\
import { nope } from react;\n";
        let call_hints = import_call_hints("src/app/main.ts", call_hints_source);
        assert!(
            call_hints.contains_key("makeBuild"),
            "valid relative named imports should produce call hints"
        );
        assert!(
            !call_hints.contains_key("bad"),
            "non-relative imports should be skipped for call hints"
        );

        assert!(
            resolve_typescript_import_paths("", "./pkg/mod").is_empty(),
            "missing base directory should return no import candidates"
        );
        assert_eq!(
            normalize_relative_path(std::path::Path::new("/tmp/./pkg/../mod.ts")),
            "tmp/mod.ts".to_string()
        );

        let blank_tree = parse_typescript_root("\n", false);
        assert_eq!(signature_summary(blank_tree.root_node(), "\n"), None);

        let tsx_tree = parse_typescript_root("const view = <div />;", true);
        assert_eq!(tsx_tree.root_node().kind(), "program");
    }

    #[test]
    fn helper_paths_cover_remaining_import_and_member_expression_guards() {
        let fallback_source = r#"
class Worker implements LocalShape {}
class Worker implements LocalShape {}
"#;
        let fallback_unit = TypeScriptLanguageAdapter
            .extract("src/app/main.ts", fallback_source)
            .expect("typescript extraction should succeed");
        assert!(
            fallback_unit.edges.iter().any(|edge| {
                edge.edge_kind == "implements"
                    && edge.to_symbol_key.symbol == "LocalShape"
                    && edge.to_symbol_key.qualified_symbol.is_none()
            }),
            "non-import implements targets should use language-level fallback keys"
        );
        assert_eq!(
            fallback_unit
                .edges
                .iter()
                .filter(|edge| {
                    edge.edge_kind == "implements"
                        && edge.from_symbol_key.symbol == "Worker"
                        && edge.to_symbol_key.symbol == "LocalShape"
                })
                .count(),
            1,
            "duplicate implements edges should deduplicate deterministically"
        );

        let call_source = "function run(){ obj.render(); }";
        let call_tree = parse_typescript_root(call_source, false);
        let member_expression = find_nodes_of_kind(call_tree.root_node(), "member_expression")
            .into_iter()
            .next()
            .expect("member expression should parse");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        collect_call_symbols(
            member_expression,
            call_source,
            Some("run"),
            "src/app/main.ts",
            "typescript",
            &HashMap::new(),
            &HashMap::new(),
            &mut references,
            &mut edges,
        );
        assert!(
            references.iter().any(|item| item.symbol == "render"),
            "member-expression fallback should recurse into property symbols"
        );
        assert!(
            edges
                .iter()
                .any(|edge| edge.to_symbol_key.symbol == "render"),
            "member-expression fallback should still emit local call edges"
        );

        let malformed_statement = "import { a as , b } from \"./x\";";
        let malformed_tree = parse_typescript_root(malformed_statement, false);
        let import_statement = find_nodes_of_kind(malformed_tree.root_node(), "import_statement")
            .into_iter()
            .next()
            .expect("import statement should parse");
        let malformed_bindings = import_bindings(import_statement, malformed_statement);
        assert!(
            malformed_bindings
                .iter()
                .any(|item| item.local_symbol == "b"),
            "valid bindings should remain even when malformed aliases are present"
        );

        let target_hints =
            import_target_hints("src/app/main.ts", "import { a as , , b } from \"./x\";\n");
        assert!(target_hints.contains_key("b"));
        assert!(!target_hints.contains_key(""));

        let call_hints = import_call_hints(
            "src/app/main.ts",
            "\
import }{ from \"./x\";\n\
import { , a, b as  } from \"./x\";\n\
import { c as makeC } from \"./x\";\n",
        );
        assert!(call_hints.contains_key("a"));
        assert!(call_hints.contains_key("makeC"));

        let blank_tree = parse_typescript_root("   \n", false);
        assert_eq!(signature_summary(blank_tree.root_node(), "   \n"), None);
    }

    #[test]
    fn collect_call_symbols_covers_identifier_and_member_expression_fallback_closing_paths() {
        let source = "function run(){ helper(); obj.render(); }";
        let tree = parse_typescript_root(source, false);
        let root = tree.root_node();
        let call_nodes = find_nodes_of_kind(root, "call_expression");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        for call in call_nodes {
            let function_node = call
                .child_by_field_name("function")
                .expect("call should expose function field");
            collect_call_symbols(
                function_node,
                source,
                Some("run"),
                "src/app/main.ts",
                "typescript",
                &HashMap::new(),
                &HashMap::new(),
                &mut references,
                &mut edges,
            );
        }
        assert!(
            references.iter().any(|item| item.symbol == "helper"),
            "identifier calls should be captured"
        );
        assert!(
            references.iter().any(|item| item.symbol == "render"),
            "member-expression property calls should be captured"
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
            "member-expression fallback should emit local call edges"
        );
    }

    #[test]
    fn collect_call_symbols_covers_empty_identifier_and_member_expression_symbol_paths() {
        let source = "function run(){ helper(); obj.render(); }";
        let tree = parse_typescript_root(source, false);
        let root = tree.root_node();
        let call_nodes = find_nodes_of_kind(root, "call_expression");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        for call in call_nodes {
            let function_node = call
                .child_by_field_name("function")
                .expect("call should expose function field");
            collect_call_symbols(
                function_node,
                "",
                Some("run"),
                "src/app/main.ts",
                "typescript",
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
}
