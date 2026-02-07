use anyhow::Context;
use tree_sitter::{Node, Parser};

use crate::indexer::languages::{
    ExtractedEdge, ExtractedReference, ExtractedSymbol, ExtractionUnit, LanguageAdapter, SymbolKey,
};

pub struct TypeScriptLanguageAdapter;

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
                            edges.push(ExtractedEdge {
                                from_symbol_key: SymbolKey {
                                    symbol: class_symbol.clone(),
                                },
                                to_symbol_key: SymbolKey {
                                    symbol: implemented,
                                },
                                edge_kind: "implements".to_string(),
                                confidence: 0.95,
                                provenance: "ast_reference".to_string(),
                            });
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
                            from_symbol_key: SymbolKey {
                                symbol: container_symbol,
                            },
                            to_symbol_key: SymbolKey { symbol },
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
                        edges.push(ExtractedEdge {
                            from_symbol_key: SymbolKey {
                                symbol: binding.local_symbol,
                            },
                            to_symbol_key: SymbolKey {
                                symbol: binding.imported_symbol,
                            },
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
                .then(left.to_symbol_key.symbol.cmp(&right.to_symbol_key.symbol))
                .then(left.edge_kind.cmp(&right.edge_kind))
        });
        edges.dedup_by(|left, right| {
            left.from_symbol_key.symbol == right.from_symbol_key.symbol
                && left.to_symbol_key.symbol == right.to_symbol_key.symbol
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
    let Some(name_node) = node.child_by_field_name("name") else {
        return None;
    };
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
    if matches!(node.kind(), "identifier" | "type_identifier")
        && let Some(symbol) = node_text(node, source)
    {
        output.push(symbol);
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_type_identifiers(child, source, output);
    }
}

fn collect_call_symbols(
    node: Node<'_>,
    source: &str,
    caller: Option<&str>,
    references: &mut Vec<ExtractedReference>,
    edges: &mut Vec<ExtractedEdge>,
) {
    match node.kind() {
        "identifier" | "property_identifier" => {
            if let Some(symbol) = node_text(node, source) {
                let (line, column) = start_position(node);
                references.push(ExtractedReference {
                    symbol: symbol.clone(),
                    line,
                    column,
                });
                if let Some(caller_symbol) = caller {
                    edges.push(ExtractedEdge {
                        from_symbol_key: SymbolKey {
                            symbol: caller_symbol.to_string(),
                        },
                        to_symbol_key: SymbolKey { symbol },
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                }
            }
        }
        "member_expression" => {
            if let Some(property) = node.child_by_field_name("property") {
                collect_call_symbols(property, source, caller, references, edges);
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_call_symbols(child, source, caller, references, edges);
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
    if line.is_empty() {
        return None;
    }
    Some(line.to_string())
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
            if imported_symbol.is_empty() || local_symbol.is_empty() {
                continue;
            }
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
