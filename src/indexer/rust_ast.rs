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
    pub caller: Option<String>,
}

/// Extracts top-level Rust item definitions and call references from the given source.
///
/// Returns a pair: a vector of discovered AST definitions (functions, structs, enums,
/// traits, modules, type aliases, consts, and imports) and a vector of call references
/// that include optional caller context.
///
/// # Examples
///
/// ```
/// let source = r#"
/// fn foo() { bar(); }
/// fn bar() {}
/// "#;
/// let (defs, refs) = extract_rust_items(source).unwrap();
/// assert!(defs.iter().any(|d| d.symbol == "foo"));
/// assert!(refs.iter().any(|r| r.symbol == "bar"));
/// ```
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
                let caller = enclosing_function_name(node, source);
                if let Some(function_node) = node.child_by_field_name("function") {
                    collect_call_identifiers(
                        function_node,
                        source,
                        caller.as_deref(),
                        &mut references,
                    );
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

/// Collect identifier symbols within a call-expression subtree and append them to `output` as `AstReference` entries.
///
/// This function walks the given `node` subtree. When it encounters an identifier node it
/// records the identifier's text, 1-based start position, and the optional `caller` into
/// `output`. For non-identifier nodes it recursively processes all children.
///
/// # Examples
///
/// ```
/// use tree_sitter::{Parser, Node};
/// use tree_sitter_rust::language;
///
/// // Minimal example wiring: parse a simple function that calls `foo`.
/// let source = "fn caller() { foo(); }";
/// let mut parser = Parser::new();
/// parser.set_language(language()).unwrap();
/// let tree = parser.parse(source, None).unwrap();
/// let root = tree.root_node();
///
/// // Find the first call_expression in the tree.
/// let mut cursor = root.walk();
/// let mut call_node: Option<Node> = None;
/// for child in root.children(&mut cursor) {
///     if child.kind() == "function_item" {
///         let mut c = child.walk();
///         for gc in child.children(&mut c) {
///             if gc.kind() == "call_expression" {
///                 call_node = Some(gc);
///                 break;
///             }
///         }
///     }
/// }
///
/// let mut refs = Vec::new();
/// if let Some(call) = call_node {
///     // Provide the enclosing function name as caller.
///     super::collect_call_identifiers(call, source, Some("caller"), &mut refs);
/// }
///
/// assert_eq!(refs.len(), 1);
/// assert_eq!(refs[0].symbol, "foo");
/// assert_eq!(refs[0].caller.as_deref(), Some("caller"));
/// ```
fn collect_call_identifiers(
    node: Node<'_>,
    source: &str,
    caller: Option<&str>,
    output: &mut Vec<AstReference>,
) {
    if node.kind() == "identifier" {
        if let Some(symbol) = node_text(node, source) {
            let (line, column) = start_position(node);
            output.push(AstReference {
                symbol,
                line,
                column,
                caller: caller.map(str::to_string),
            });
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_call_identifiers(child, source, caller, output);
    }
}

/// Create an `AstDefinition` from the node's declared name and append it to `output`.
///
/// If the node has no `name` field, if the name is not an `identifier` or `type_identifier`,
/// or if the name's text cannot be extracted from `source`, the function returns without modifying `output`.
///
/// # Examples
///
/// ```
/// // given a parsed `node` for a function or type and the original `source`
/// let mut defs = Vec::new();
/// // push_named_definition(node, source, "function", None, Some("fn foo()".to_string()), &mut defs);
/// // assert!(defs.iter().any(|d| d.kind == "function"));
/// ```
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
    let (end_line, end_column) = end_position(node);
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

/// Finds the type name for the `impl` block that encloses `node`, if any.
///
/// The returned `String` is the last identifier text of the `impl`'s `type` child
/// (for example, for `impl Foo { ... }` this returns `"Foo"`). Returns `None` if
/// the node is not inside an `impl` block or the `impl` has no identifiable type.
///
/// # Examples
///
/// ```
/// use tree_sitter::{Parser, Node};
/// // Ensure the `tree-sitter-rust` crate is available in Cargo.toml for this example:
/// // tree-sitter-rust = "0.20"
/// let source = r#"
/// impl Foo {
///     fn bar(&self) {}
/// }
/// "#;
///
/// let mut parser = Parser::new();
/// tree_sitter_rust::language().and_then(|lang| parser.set_language(lang)).expect("language loaded");
/// let tree = parser.parse(source, None).expect("parsed");
/// let root = tree.root_node();
///
/// // Find the first `function_item` node.
/// let mut stack = vec![root];
/// let mut fn_node: Option<Node<'_>> = None;
/// while let Some(n) = stack.pop() {
///     if n.kind() == "function_item" {
///         fn_node = Some(n);
///         break;
///     }
///     for i in 0..n.child_count() {
///         if let Some(child) = n.child(i) {
///             stack.push(child);
///         }
///     }
/// }
///
/// let container = fn_node
///     .and_then(|n| enclosing_impl_container(n, source));
/// assert_eq!(container.as_deref(), Some("Foo"));
/// ```
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

/// Returns the text of the last `identifier` or `type_identifier` found in the given node's subtree.
///
/// Traverses the node's descendants in document order and returns the text of the final identifier-type node encountered,
/// or `None` if no identifier or type identifier exists in the subtree.
///
/// # Examples
///
/// ```
/// use tree_sitter::{Parser, Node};
/// use tree_sitter_rust::language;
///
/// // Parse a small Rust snippet and find the last identifier inside the function body.
/// let mut parser = Parser::new();
/// parser.set_language(language()).unwrap();
/// let source = "fn foo() { let x = some::call(); }";
/// let tree = parser.parse(source, None).unwrap();
/// let root = tree.root_node();
///
/// // Find the first function_item and get its body node to search within.
/// let mut cursor = root.walk();
/// let mut func_node: Option<Node> = None;
/// for child in root.children(&mut cursor) {
///     if child.kind() == "function_item" {
///         func_node = Some(child);
///         break;
///     }
/// }
///
/// if let Some(f) = func_node {
///     // Search the function node's subtree for the last identifier text.
///     let text = crate::last_identifier_text(f, source);
///     // The last identifier in the snippet is "call".
///     assert_eq!(text.as_deref(), Some("call"));
/// }
/// ```
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

/// Extracts the UTF-8 text slice of `node` from the provided source string.
///
/// Returns `Some(String)` containing the node's source text when the node's
/// byte range is within `source` and the bytes form valid UTF-8, or `None` if
/// the range is out of bounds or the bytes are not valid UTF-8.
///
/// # Examples
///
/// ```no_run
/// use tree_sitter::Parser;
/// // parse some Rust source and obtain a `Node` (omitted for brevity)
/// let source = "fn foo() {}";
/// let mut parser = Parser::new();
/// parser.set_language(tree_sitter_rust::language()).unwrap();
/// let tree = parser.parse(source, None).unwrap();
/// let root = tree.root_node();
/// // find an identifier node (example only)
/// if let Some(ident) = root.descendant_for_byte_range(3, 6) {
///     let text = node_text(ident, source);
///     assert_eq!(text.as_deref(), Some("foo"));
/// }
/// ```
fn node_text(node: Node<'_>, source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let span = bytes.get(node.start_byte()..node.end_byte())?;
    std::str::from_utf8(span).ok().map(str::to_string)
}

/// Convert a node's 0-based start point to 1-based (line, column) coordinates.
///
/// The returned tuple is (line, column) where both values start at 1.
///
/// # Examples
///
/// ```
/// use tree_sitter::Parser;
/// // parse a tiny Rust snippet and get the root node
/// let mut parser = Parser::new();
/// parser.set_language(tree_sitter_rust::language()).unwrap();
/// let source = "fn main() {}\n";
/// let tree = parser.parse(source, None).unwrap();
/// let root = tree.root_node();
///
/// let pos = start_position(root);
/// assert_eq!(pos, (1, 1));
/// ```
fn start_position(node: Node<'_>) -> (u32, u32) {
    let point = node.start_position();
    (point.row as u32 + 1, point.column as u32 + 1)
}

/// Convert a node's end position to 1-based (line, column) coordinates.
///
/// The returned tuple is (line, column), where both values start at 1.
///
/// # Examples
///
/// ```
/// use tree_sitter::Parser;
/// use tree_sitter_rust::language;
/// // Build a simple parse tree for a single function
/// let mut parser = Parser::new();
/// parser.set_language(language()).unwrap();
/// let tree = parser.parse("fn f() {}", None).unwrap();
/// let root = tree.root_node();
/// let fn_node = root.named_child(0).unwrap(); // the function_item node
/// let pos = end_position(fn_node);
/// assert!(pos.0 >= 1 && pos.1 >= 1);
/// ```
fn end_position(node: Node<'_>) -> (u32, u32) {
    let point = node.end_position();
    (point.row as u32 + 1, point.column as u32 + 1)
}

/// Produces a short signature string for a syntax node by taking its first line
/// and trimming any trailing block opener.
///
/// # Returns
///
/// `Some(String)` containing the node's first line trimmed and truncated before
/// the first `{`, or `None` if the node has no text or the resulting string is empty.
///
/// # Examples
///
/// ```
/// use tree_sitter::Parser;
/// use tree_sitter_rust::language;
///
/// let src = "fn add(a: i32) -> i32 { a + 1 }\n";
/// let mut parser = Parser::new();
/// parser.set_language(language()).unwrap();
/// let tree = parser.parse(src, None).unwrap();
/// let root = tree.root_node();
/// // find the first child node that represents the function item
/// let func_node = root.child(0).unwrap();
/// let sig = signature_summary(func_node, src);
/// assert_eq!(sig, Some("fn add(a: i32) -> i32".to_string()));
/// ```
fn signature_summary(node: Node<'_>, source: &str) -> Option<String> {
    let text = node_text(node, source)?;
    let first_line = text.lines().next()?.trim();
    let head = first_line.split('{').next()?.trim();
    if head.is_empty() {
        return None;
    }
    Some(head.to_string())
}

/// Finds the name of the nearest enclosing Rust function for the given node.
///
/// Searches upward from `node` through its ancestors and returns the text of the first
/// ancestor whose kind is `function_item`. Returns `None` if no enclosing function is found.
///
/// # Examples
///
/// ```
/// use tree_sitter::Parser;
/// use tree_sitter_rust::language;
/// // assume enclosing_function_name is in scope
///
/// let source = r#"fn foo() { bar(); }"#;
/// let mut parser = Parser::new();
/// parser.set_language(language()).unwrap();
/// let tree = parser.parse(source, None).unwrap();
/// let root = tree.root_node();
///
/// // find any identifier inside the function body (the call target `bar`)
/// let mut stack = vec![root];
/// let mut target_node = None;
/// while let Some(node) = stack.pop() {
///     if node.kind() == "identifier" {
///         target_node = Some(node);
///         break;
///     }
///     for i in 0..node.child_count() {
///         if let Some(child) = node.child(i) { stack.push(child); }
///     }
/// }
///
/// let node = target_node.expect("expected an identifier in the source");
/// let name = enclosing_function_name(node, source);
/// assert_eq!(name.as_deref(), Some("foo"));
/// ```
fn enclosing_function_name(node: Node<'_>, source: &str) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            let name_node = parent.child_by_field_name("name")?;
            return node_text(name_node, source);
        }
        current = parent.parent();
    }
    None
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
