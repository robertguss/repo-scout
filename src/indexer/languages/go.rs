use anyhow::Context;
use tree_sitter::{Node, Parser};

use crate::indexer::languages::{ExtractedSymbol, ExtractionUnit, LanguageAdapter};

pub struct GoLanguageAdapter;

impl LanguageAdapter for GoLanguageAdapter {
    fn language_id(&self) -> &'static str {
        "go"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["go"]
    }

    fn extract(&self, file_path: &str, source: &str) -> anyhow::Result<ExtractionUnit> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .context("failed to load tree-sitter go language")?;

        let tree = parser
            .parse(source, None)
            .context("failed to parse go source")?;

        let language = self.language_id().to_string();
        let mut symbols = Vec::new();
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
                "method_declaration" => {
                    push_named_definition(
                        node,
                        source,
                        "method",
                        method_receiver_name(node, source),
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "type_spec" => {
                    push_named_definition(
                        node,
                        source,
                        "type",
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "const_spec" => {
                    push_identifier_list_definitions(
                        node,
                        source,
                        "const",
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "var_spec" => {
                    push_identifier_list_definitions(
                        node,
                        source,
                        "variable",
                        file_path,
                        &language,
                        &mut symbols,
                    );
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
        symbols.dedup_by(|left, right| {
            left.symbol == right.symbol
                && left.kind == right.kind
                && left.start_line == right.start_line
                && left.start_column == right.start_column
        });

        Ok(ExtractionUnit {
            symbols,
            ..ExtractionUnit::default()
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
    if !is_name_node(name_node.kind()) {
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
        signature: None,
    });

    Some(symbol)
}

fn push_identifier_list_definitions(
    node: Node<'_>,
    source: &str,
    kind: &str,
    file_path: &str,
    language: &str,
    output: &mut Vec<ExtractedSymbol>,
) {
    let Some(names_node) = node.child_by_field_name("name") else {
        return;
    };
    let mut stack = vec![names_node];
    while let Some(current) = stack.pop() {
        if is_name_node(current.kind()) {
            if let Some(symbol) = node_text(current, source) {
                let (start_line, start_column) = start_position(current);
                let (end_line, end_column) = end_position(node);
                output.push(ExtractedSymbol {
                    symbol: symbol.clone(),
                    qualified_symbol: Some(format!("{language}:{file_path}::{symbol}")),
                    kind: kind.to_string(),
                    language: language.to_string(),
                    container: None,
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                    signature: None,
                });
            }
            continue;
        }
        let mut cursor = current.walk();
        for child in current.children(&mut cursor) {
            stack.push(child);
        }
    }
}

fn method_receiver_name(node: Node<'_>, source: &str) -> Option<String> {
    let receiver_node = node.child_by_field_name("receiver")?;
    last_identifier_text(receiver_node, source)
}

fn last_identifier_text(node: Node<'_>, source: &str) -> Option<String> {
    let mut last = None;
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if is_name_node(current.kind()) {
            last = node_text(current, source);
            continue;
        }
        let mut cursor = current.walk();
        let children: Vec<_> = current.children(&mut cursor).collect();
        for child in children.into_iter().rev() {
            stack.push(child);
        }
    }
    last
}

fn is_name_node(kind: &str) -> bool {
    matches!(kind, "identifier" | "field_identifier" | "type_identifier")
}

fn node_text(node: Node<'_>, source: &str) -> Option<String> {
    node.utf8_text(source.as_bytes())
        .ok()
        .map(str::trim)
        .filter(|text| !text.is_empty())
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
