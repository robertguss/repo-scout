use anyhow::Context;
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone)]
pub struct AstDefinition {
    pub symbol: String,
    pub kind: String,
    pub container: Option<String>,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub signature: Option<String>,
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
                push_named_definition(
                    node,
                    source,
                    "function",
                    enclosing_impl_container(node, source),
                    signature_summary(node, source),
                    &mut definitions,
                );
            }
            "struct_item" => {
                push_named_definition(node, source, "struct", None, None, &mut definitions);
            }
            "enum_item" => {
                push_named_definition(node, source, "enum", None, None, &mut definitions);
            }
            "trait_item" => {
                push_named_definition(node, source, "trait", None, None, &mut definitions);
            }
            "mod_item" => {
                push_named_definition(node, source, "module", None, None, &mut definitions);
            }
            "type_item" => {
                push_named_definition(node, source, "type_alias", None, None, &mut definitions);
            }
            "const_item" => {
                push_named_definition(node, source, "const", None, None, &mut definitions);
            }
            "use_declaration" => {
                if let Some(symbol) = last_identifier_text(node, source) {
                    let (line, column) = start_position(node);
                    let (end_line, end_column) = end_position(node);
                    definitions.push(AstDefinition {
                        symbol,
                        kind: "import".to_string(),
                        container: None,
                        line,
                        column,
                        end_line,
                        end_column,
                        signature: None,
                    });
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

fn push_named_definition(
    node: Node<'_>,
    source: &str,
    kind: &str,
    container: Option<String>,
    signature: Option<String>,
    output: &mut Vec<AstDefinition>,
) {
    let Some(name_node) = node.child_by_field_name("name") else {
        return;
    };
    if !matches!(name_node.kind(), "identifier" | "type_identifier") {
        return;
    }

    let Some(symbol) = node_text(name_node, source) else {
        return;
    };

    let (line, column) = start_position(name_node);
    let (end_line, end_column) = end_position(name_node);
    output.push(AstDefinition {
        symbol,
        kind: kind.to_string(),
        container,
        line,
        column,
        end_line,
        end_column,
        signature,
    });
}

fn enclosing_impl_container(node: Node<'_>, source: &str) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "impl_item" {
            if let Some(type_node) = parent.child_by_field_name("type") {
                return last_identifier_text(type_node, source);
            }
        }
        current = parent.parent();
    }
    None
}

fn last_identifier_text(node: Node<'_>, source: &str) -> Option<String> {
    if matches!(node.kind(), "identifier" | "type_identifier") {
        return node_text(node, source);
    }

    let mut cursor = node.walk();
    let mut last = None;
    for child in node.children(&mut cursor) {
        if let Some(value) = last_identifier_text(child, source) {
            last = Some(value);
        }
    }
    last
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

fn end_position(node: Node<'_>) -> (u32, u32) {
    let point = node.end_position();
    (point.row as u32 + 1, point.column as u32 + 1)
}

fn signature_summary(node: Node<'_>, source: &str) -> Option<String> {
    let text = node_text(node, source)?;
    let first_line = text.lines().next()?.trim();
    let head = first_line.split('{').next()?.trim();
    if head.is_empty() {
        return None;
    }
    Some(head.to_string())
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
