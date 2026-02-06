use anyhow::Context;
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone)]
pub struct AstDefinition {
    pub symbol: String,
    pub kind: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub struct AstReference {
    pub symbol: String,
    pub line: u32,
    pub column: u32,
}

pub fn extract_rust_items(source: &str) -> anyhow::Result<(Vec<AstDefinition>, Vec<AstReference>)> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .context("failed to load tree-sitter rust language")?;

    let tree = parser
        .parse(source, None)
        .context("failed to parse rust source")?;

    let mut definitions = Vec::new();
    let mut references = Vec::new();
    let mut stack = vec![tree.root_node()];

    while let Some(node) = stack.pop() {
        match node.kind() {
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if name_node.kind() == "identifier" {
                        if let Some(symbol) = node_text(name_node, source) {
                            let (line, column) = start_position(name_node);
                            definitions.push(AstDefinition {
                                symbol,
                                kind: "function".to_string(),
                                line,
                                column,
                            });
                        }
                    }
                }
            }
            "call_expression" => {
                if let Some(function_node) = node.child_by_field_name("function") {
                    collect_call_identifiers(function_node, source, &mut references);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }

    definitions.sort_by(|left, right| {
        left.file_order_key()
            .cmp(&right.file_order_key())
            .then(left.symbol.cmp(&right.symbol))
    });
    references.sort_by(|left, right| {
        left.file_order_key()
            .cmp(&right.file_order_key())
            .then(left.symbol.cmp(&right.symbol))
    });

    Ok((definitions, references))
}

fn collect_call_identifiers(node: Node<'_>, source: &str, output: &mut Vec<AstReference>) {
    if node.kind() == "identifier" {
        if let Some(symbol) = node_text(node, source) {
            let (line, column) = start_position(node);
            output.push(AstReference {
                symbol,
                line,
                column,
            });
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_call_identifiers(child, source, output);
    }
}

fn node_text(node: Node<'_>, source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let span = bytes.get(node.start_byte()..node.end_byte())?;
    std::str::from_utf8(span).ok().map(str::to_string)
}

fn start_position(node: Node<'_>) -> (u32, u32) {
    let point = node.start_position();
    (point.row as u32 + 1, point.column as u32 + 1)
}

trait FileOrderKey {
    fn file_order_key(&self) -> (u32, u32);
}

impl FileOrderKey for AstDefinition {
    fn file_order_key(&self) -> (u32, u32) {
        (self.line, self.column)
    }
}

impl FileOrderKey for AstReference {
    fn file_order_key(&self) -> (u32, u32) {
        (self.line, self.column)
    }
}
