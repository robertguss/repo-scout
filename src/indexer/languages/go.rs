use anyhow::Context;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tree_sitter::{Node, Parser};

use crate::indexer::languages::{
    ExtractedEdge, ExtractedReference, ExtractedSymbol, ExtractionUnit, LanguageAdapter, SymbolKey,
};

pub struct GoLanguageAdapter;

#[derive(Debug, Clone)]
struct GoImportBinding {
    local_symbol: String,
    imported_symbol: String,
    import_path: String,
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
        let import_target_hints = import_target_hints(file_path, source, tree.root_node());
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
                "method_declaration" => {
                    let container = method_receiver_name(node, source);
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
                "type_spec" => {
                    push_named_definition(
                        node,
                        source,
                        go_type_kind(node),
                        None,
                        file_path,
                        &language,
                        &mut symbols,
                    );
                }
                "type_alias" => {
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
                "import_spec" => {
                    if let Some(binding) = import_binding(node, source, file_path) {
                        let (start_line, start_column) = start_position(node);
                        let (end_line, end_column) = end_position(node);
                        symbols.push(ExtractedSymbol {
                            symbol: binding.local_symbol.clone(),
                            qualified_symbol: Some(format!(
                                "{language}:{file_path}::{}",
                                binding.local_symbol
                            )),
                            kind: "import".to_string(),
                            language: language.clone(),
                            container: None,
                            start_line,
                            start_column,
                            end_line,
                            end_column,
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
                            &mut references,
                            &mut edges,
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
    let Some(name_node) = node.child_by_field_name("name") else {
        return;
    };
    if !is_name_node(name_node.kind()) {
        return;
    }
    if let Some(symbol) = node_text(name_node, source) {
        let (start_line, start_column) = start_position(name_node);
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
}

fn method_receiver_name(node: Node<'_>, source: &str) -> Option<String> {
    let receiver_node = node.child_by_field_name("receiver")?;
    last_identifier_text(receiver_node, source)
}

fn go_type_kind(node: Node<'_>) -> &'static str {
    let Some(type_node) = node.child_by_field_name("type") else {
        return "type";
    };
    match type_node.kind() {
        "interface_type" => "interface",
        "struct_type" => "struct",
        _ => "type",
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_call_symbols(
    node: Node<'_>,
    source: &str,
    caller: Option<&str>,
    file_path: &str,
    language: &str,
    import_target_hints: &HashMap<String, String>,
    references: &mut Vec<ExtractedReference>,
    edges: &mut Vec<ExtractedEdge>,
) {
    match node.kind() {
        "identifier" | "field_identifier" => {
            let symbol = node_text(node, source).unwrap_or_default();
            if symbol.is_empty() {
                return;
            }
            let (line, column) = start_position(node);
            references.push(ExtractedReference {
                symbol: symbol.clone(),
                line,
                column,
            });
            if let Some(caller_symbol) = caller {
                for to_symbol_key in
                    call_target_symbol_keys(file_path, language, &symbol, None, import_target_hints)
                {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                        to_symbol_key,
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                }
            }
        }
        "selector_expression" => {
            let qualifier = node
                .child_by_field_name("operand")
                .and_then(|operand| node_text(operand, source))
                .and_then(|text| first_identifier(&text));
            let field_node = node.child_by_field_name("field").unwrap_or(node);
            let symbol = node_text(field_node, source).unwrap_or_default();
            if symbol.is_empty() {
                return;
            }
            let (line, column) = start_position(field_node);
            references.push(ExtractedReference {
                symbol: symbol.clone(),
                line,
                column,
            });

            if let Some(caller_symbol) = caller {
                for to_symbol_key in call_target_symbol_keys(
                    file_path,
                    language,
                    &symbol,
                    qualifier.as_deref(),
                    import_target_hints,
                ) {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, caller_symbol),
                        to_symbol_key,
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                }
            }
            return;
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_call_symbols(
            child,
            source,
            caller,
            file_path,
            language,
            import_target_hints,
            references,
            edges,
        );
    }
}

fn call_target_symbol_keys(
    caller_file_path: &str,
    language: &str,
    symbol: &str,
    qualifier: Option<&str>,
    import_target_hints: &HashMap<String, String>,
) -> Vec<SymbolKey> {
    let mut keys = Vec::new();

    if let Some(qualifier) = qualifier
        && let Some(import_path) = import_target_hints.get(qualifier)
    {
        for candidate_file in go_import_candidate_paths(import_path) {
            keys.push(SymbolKey {
                symbol: symbol.to_string(),
                qualified_symbol: Some(format!("{language}:{candidate_file}::{symbol}")),
                file_path: Some(candidate_file),
                language: Some(language.to_string()),
            });
        }
    }

    keys.push(SymbolKey {
        symbol: symbol.to_string(),
        qualified_symbol: None,
        file_path: Some(caller_file_path.to_string()),
        language: Some(language.to_string()),
    });
    keys.push(language_symbol_key(symbol, language));

    dedupe_symbol_keys(keys)
}

fn go_import_candidate_paths(import_path: &str) -> Vec<String> {
    if import_path.ends_with(".go") {
        return vec![import_path.to_string()];
    }

    let trimmed = import_path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut candidates = vec![format!("{trimmed}.go")];
    if let Some(stem) = trimmed.rsplit('/').next()
        && !stem.is_empty()
    {
        candidates.push(format!("{trimmed}/{stem}.go"));
    }
    candidates.push(format!("{trimmed}/main.go"));

    let mut deduped = Vec::new();
    let mut seen = HashSet::new();
    for candidate in candidates {
        if seen.insert(candidate.clone()) {
            deduped.push(candidate);
        }
    }
    deduped
}

fn dedupe_symbol_keys(keys: Vec<SymbolKey>) -> Vec<SymbolKey> {
    let mut deduped = Vec::new();
    let mut seen = HashSet::new();
    for key in keys {
        let unique_key = (
            key.symbol.clone(),
            key.qualified_symbol.clone(),
            key.file_path.clone(),
            key.language.clone(),
        );
        if seen.insert(unique_key) {
            deduped.push(key);
        }
    }
    deduped
}

fn import_target_hints(file_path: &str, source: &str, root: Node<'_>) -> HashMap<String, String> {
    let mut hints = HashMap::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "import_spec"
            && let Some(binding) = import_binding(node, source, file_path)
        {
            hints.insert(binding.local_symbol, binding.import_path);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    hints
}

fn import_binding(node: Node<'_>, source: &str, file_path: &str) -> Option<GoImportBinding> {
    let path_node = node.child_by_field_name("path")?;
    let import_literal = node_text(path_node, source)?;
    let import_specifier = unquote_go_string(&import_literal)?;
    let import_path = resolve_go_import_path(file_path, &import_specifier)?;

    let imported_symbol = package_name_from_import(&import_specifier)?;
    let local_symbol = match node.child_by_field_name("name") {
        Some(name_node) => match name_node.kind() {
            "blank_identifier" | "dot" => return None,
            _ => node_text(name_node, source)?,
        },
        None => imported_symbol.clone(),
    };

    Some(GoImportBinding {
        local_symbol,
        imported_symbol,
        import_path,
    })
}

fn resolve_go_import_path(from_file_path: &str, import_specifier: &str) -> Option<String> {
    let trimmed = import_specifier.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('.') {
        let base = Path::new(from_file_path).parent()?;
        let candidate = base.join(trimmed);
        return Some(normalize_relative_path(&candidate));
    }

    let normalized = trimmed.trim_matches('/');
    if normalized.is_empty() {
        return None;
    }

    let first_component = Path::new(from_file_path)
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .filter(|component| !component.is_empty());

    if let Some(prefix) = first_component {
        if normalized.starts_with(&format!("{prefix}/")) {
            Some(normalized.to_string())
        } else {
            Some(format!("{prefix}/{normalized}"))
        }
    } else {
        Some(normalized.to_string())
    }
}

fn normalize_relative_path(path: &Path) -> String {
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

fn package_name_from_import(import_specifier: &str) -> Option<String> {
    import_specifier
        .trim_matches('/')
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .map(str::to_string)
}

fn unquote_go_string(literal: &str) -> Option<String> {
    let trimmed = literal.trim();
    if trimmed.len() < 2 {
        return None;
    }
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('`') && trimmed.ends_with('`'))
    {
        return Some(trimmed[1..trimmed.len() - 1].to_string());
    }
    None
}

fn enclosing_callable_name(node: Node<'_>, source: &str) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if matches!(parent.kind(), "function_declaration" | "method_declaration")
            && let Some(name_node) = parent.child_by_field_name("name")
        {
            return node_text(name_node, source);
        }
        current = parent.parent();
    }
    None
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

fn first_identifier(text: &str) -> Option<String> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|part| !part.is_empty())
        .map(str::to_string)
}

fn is_name_node(kind: &str) -> bool {
    matches!(kind, "identifier" | "field_identifier" | "type_identifier")
}

fn node_text(node: Node<'_>, source: &str) -> Option<String> {
    source
        .as_bytes()
        .get(node.start_byte()..node.end_byte())
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_go_root(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("go language should load");
        parser.parse(source, None).expect("go source should parse")
    }

    fn find_first_kind<'a>(root: Node<'a>, kind: &str) -> Option<Node<'a>> {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == kind {
                return Some(node);
            }
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor).collect::<Vec<_>>();
            children.reverse();
            for child in children {
                stack.push(child);
            }
        }
        None
    }

    #[test]
    fn go_import_and_path_helpers_cover_edge_cases() {
        assert_eq!(
            go_import_candidate_paths("pkg/service.go"),
            vec!["pkg/service.go".to_string()]
        );
        assert!(go_import_candidate_paths("").is_empty());
        assert_eq!(
            go_import_candidate_paths("pkg/service"),
            vec![
                "pkg/service.go".to_string(),
                "pkg/service/service.go".to_string(),
                "pkg/service/main.go".to_string()
            ]
        );

        assert_eq!(resolve_go_import_path("src/app/main.go", "  "), None);
        assert_eq!(resolve_go_import_path("src/app/main.go", "/"), None);
        assert_eq!(
            resolve_go_import_path("src/app/main.go", "src/pkg/tool"),
            Some("src/pkg/tool".to_string())
        );
        assert_eq!(
            resolve_go_import_path("src/app/main.go", "pkg/tool"),
            Some("src/pkg/tool".to_string())
        );
        assert_eq!(
            resolve_go_import_path("main.go", "pkg/tool"),
            Some("main.go/pkg/tool".to_string())
        );
        assert_eq!(
            resolve_go_import_path("", "pkg/tool"),
            Some("pkg/tool".to_string())
        );

        assert_eq!(unquote_go_string("x"), None);
        assert_eq!(
            unquote_go_string("\"pkg/tool\""),
            Some("pkg/tool".to_string())
        );
        assert_eq!(
            unquote_go_string("`pkg/tool`"),
            Some("pkg/tool".to_string())
        );
        assert_eq!(unquote_go_string("'pkg/tool'"), None);
        assert_eq!(
            package_name_from_import("/pkg/tool/"),
            Some("tool".to_string())
        );

        assert_eq!(
            normalize_relative_path(Path::new("src/pkg/../util/./tool.go")),
            "src/util/tool.go".to_string()
        );
    }

    #[test]
    fn adapter_extract_covers_const_var_import_and_call_paths() {
        let source = r#"
package main

import (
    alias "pkg/tools"
    . "pkg/dot"
    _ "pkg/blank"
    "pkg/noalias"
)

type Worker struct{}
type Name = string
type Greeter interface{ Hello() }

const (
    CONST_A = 1
    CONST_B = 2
)

var (
    varA = 1
    varB = 2
)

func (w *Worker) run() {
    alias.Do()
    noalias.Call()
    helper()
}

func helper() {}
"#;

        let unit = GoLanguageAdapter
            .extract("src/main.go", source)
            .expect("extraction should succeed");

        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "const" && item.symbol == "CONST_A"),
            "const_spec should emit const symbols"
        );
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "variable" && item.symbol == "varA"),
            "var_spec should emit variable symbols"
        );
        assert!(
            unit.symbols
                .iter()
                .any(|item| item.kind == "import" && item.symbol == "alias"),
            "import bindings should emit import symbols"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "contains"
                    && edge.from_symbol_key.symbol == "Worker"
                    && edge.to_symbol_key.symbol == "run"
            }),
            "method container relation should produce contains edge"
        );
        assert!(
            unit.references
                .iter()
                .any(|reference| reference.symbol == "Do" || reference.symbol == "Call"),
            "selector calls should emit references"
        );
    }

    #[test]
    fn named_definition_and_call_helpers_cover_none_branches() {
        let source = r#"
package main
import _ "pkg/x"
func run() { helper() }
"#;
        let tree = parse_go_root(source);
        let root = tree.root_node();
        let import_spec = find_first_kind(root, "import_spec").expect("import_spec should exist");
        let call_expression =
            find_first_kind(root, "call_expression").expect("call_expression should exist");

        let mut output = Vec::new();
        assert_eq!(
            push_named_definition(
                import_spec,
                source,
                "function",
                None,
                "src/main.go",
                "go",
                &mut output
            ),
            None
        );
        assert!(output.is_empty());
        push_identifier_list_definitions(
            import_spec,
            source,
            "import",
            "src/main.go",
            "go",
            &mut output,
        );
        assert!(
            output.is_empty(),
            "non-identifier import name bindings should short-circuit identifier-list extraction"
        );

        let function_node = call_expression
            .child_by_field_name("function")
            .expect("function node should exist");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        collect_call_symbols(
            function_node,
            source,
            None,
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut references,
            &mut edges,
        );
        assert!(
            !references.is_empty(),
            "recursive call collection should capture symbols even without caller"
        );
        assert!(edges.is_empty(), "no caller means no call edges");
    }

    #[test]
    fn callable_and_identifier_helpers_cover_fallbacks() {
        let source = "package main\n\nvar x = 1\n";
        let tree = parse_go_root(source);
        let root = tree.root_node();
        assert_eq!(enclosing_callable_name(root, source), None);
        assert_eq!(last_identifier_text(root, source), Some("x".to_string()));
        assert_eq!(first_identifier("..."), None);
        assert!(is_name_node("identifier"));
        assert!(!is_name_node("literal"));

        let var_spec = find_first_kind(root, "var_spec").expect("var_spec should exist");
        assert_eq!(node_text(var_spec, source), Some("x = 1".to_string()));
        let (start_line, start_column) = start_position(var_spec);
        let (end_line, end_column) = end_position(var_spec);
        assert!(start_line >= 1 && start_column >= 1);
        assert!(end_line >= start_line && end_column >= 1);
    }

    #[test]
    fn helper_paths_cover_dedup_recursion_and_missing_field_fallbacks() {
        let source = r#"
package main

import alias "pkg/tools"

type Alias string

var (
    a, b = 1, 2
)

func dupe() {}
func dupe() {}

func run() {
    alias.Do()
    helper()
}
"#;
        let unit = GoLanguageAdapter
            .extract("src/main.go", source)
            .expect("extraction should succeed");
        assert!(
            unit.symbols.iter().any(|item| item.symbol == "dupe"),
            "duplicate definitions should still leave one symbol after dedup"
        );

        let tree = parse_go_root(source);
        let root = tree.root_node();
        assert!(
            find_first_kind(root, "missing_kind").is_none(),
            "missing kinds should return None from finder helper"
        );

        let mut defs = Vec::new();
        let var_spec = find_first_kind(root, "var_spec").expect("var_spec should exist");
        push_identifier_list_definitions(
            var_spec,
            source,
            "variable",
            "src/main.go",
            "go",
            &mut defs,
        );
        assert!(
            defs.iter().any(|item| item.symbol == "a"),
            "identifier list recursion should emit variable symbols"
        );

        let mut missing_name_defs = Vec::new();
        push_identifier_list_definitions(
            root,
            source,
            "variable",
            "src/main.go",
            "go",
            &mut missing_name_defs,
        );
        assert!(
            missing_name_defs.is_empty(),
            "nodes without a name field should return early"
        );

        assert_eq!(go_type_kind(root), "type");
        let type_spec = find_first_kind(root, "type_spec").expect("type_spec should exist");
        assert_eq!(go_type_kind(type_spec), "type");

        let mut refs = Vec::new();
        let mut edges = Vec::new();
        let identifier = find_first_kind(root, "identifier").expect("identifier should exist");
        collect_call_symbols(
            identifier,
            source,
            None,
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut refs,
            &mut edges,
        );
        assert!(
            !refs.is_empty() && edges.is_empty(),
            "identifier calls without caller context should record references only"
        );

        let selector =
            find_first_kind(root, "selector_expression").expect("selector expression should exist");
        collect_call_symbols(
            selector,
            source,
            Some("run"),
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut refs,
            &mut edges,
        );
        assert!(
            !edges.is_empty(),
            "selector calls with caller context should emit call edges"
        );

        let mut recursive_refs = Vec::new();
        let mut recursive_edges = Vec::new();
        collect_call_symbols(
            root,
            source,
            Some("run"),
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut recursive_refs,
            &mut recursive_edges,
        );
        assert!(
            !recursive_refs.is_empty(),
            "fallback recursion should walk child nodes and collect references"
        );

        assert_eq!(
            normalize_relative_path(Path::new("/tmp/./pkg/../tool.go")),
            "tmp/tool.go".to_string()
        );
    }

    #[test]
    fn helper_paths_cover_remaining_dedup_and_selector_return_paths() {
        let duplicate_symbol_source = r#"
package main
func dupe() {} func dupe() {}
"#;
        let duplicate_unit = GoLanguageAdapter
            .extract("src/main.go", duplicate_symbol_source)
            .expect("go extraction should succeed");
        assert!(
            duplicate_unit
                .symbols
                .iter()
                .filter(|item| item.kind == "function" && item.symbol == "dupe")
                .count()
                >= 2,
            "same-line duplicate function names should both be retained before sorting"
        );

        let var_list_source = r#"
package main
var a, b = 1, 2
"#;
        let tree = parse_go_root(var_list_source);
        let root = tree.root_node();
        let var_spec = find_first_kind(root, "var_spec").expect("var_spec should exist");
        let mut var_defs = Vec::new();
        push_identifier_list_definitions(
            var_spec,
            var_list_source,
            "variable",
            "src/main.go",
            "go",
            &mut var_defs,
        );
        assert!(
            !var_defs.is_empty(),
            "identifier-list helper should emit symbols for name fields"
        );

        let call_source = r#"
package main
func run() {
    alias.Do()
}
"#;
        let call_tree = parse_go_root(call_source);
        let call_root = call_tree.root_node();
        let selector = find_first_kind(call_root, "selector_expression")
            .expect("selector expression should exist");
        let mut selector_refs = Vec::new();
        let mut selector_edges = Vec::new();
        collect_call_symbols(
            selector,
            call_source,
            Some("run"),
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut selector_refs,
            &mut selector_edges,
        );
        assert!(
            selector_refs.iter().any(|item| item.symbol == "Do"),
            "selector expressions should capture field references"
        );
        assert!(
            selector_edges
                .iter()
                .any(|edge| edge.to_symbol_key.symbol == "Do"),
            "selector-expression branch should emit call edges and return early"
        );

        let identifier = find_first_kind(call_root, "identifier").expect("identifier should exist");
        let mut identifier_refs = Vec::new();
        let mut identifier_edges = Vec::new();
        collect_call_symbols(
            identifier,
            call_source,
            Some("run"),
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut identifier_refs,
            &mut identifier_edges,
        );
        assert!(
            !identifier_refs.is_empty() && !identifier_edges.is_empty(),
            "identifier call branch should record both references and edges"
        );
    }

    #[test]
    fn helper_paths_cover_identifier_list_recursion_and_selector_without_caller() {
        let source = r#"
package main

func consume(first, second int) {}

func run() {
    alias.Do()
}
"#;
        let tree = parse_go_root(source);
        let root = tree.root_node();
        let parameter_declaration = find_first_kind(root, "parameter_declaration")
            .expect("parameter declaration should exist");
        let mut defs = Vec::new();
        push_identifier_list_definitions(
            parameter_declaration,
            source,
            "parameter",
            "src/main.go",
            "go",
            &mut defs,
        );
        assert!(
            defs.iter().any(|item| item.symbol == "first"),
            "identifier-list recursion should emit the first parameter name"
        );

        let selector =
            find_first_kind(root, "selector_expression").expect("selector expression should exist");
        let mut references = Vec::new();
        let mut edges = Vec::new();
        collect_call_symbols(
            selector,
            source,
            None,
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut references,
            &mut edges,
        );
        assert!(
            references.iter().any(|item| item.symbol == "Do"),
            "selector-expression paths should still collect field references"
        );
        assert!(
            edges.is_empty(),
            "selector-expression branch should return early without caller context"
        );
    }

    #[test]
    fn collect_call_symbols_guards_cover_empty_symbol_and_missing_selector_field_paths() {
        let identifier_source = r#"
package main
func run() { helper() }
"#;
        let identifier_tree = parse_go_root(identifier_source);
        let identifier_call = find_first_kind(identifier_tree.root_node(), "call_expression")
            .expect("identifier call expression should exist");
        let identifier_function = identifier_call
            .child_by_field_name("function")
            .expect("identifier call should expose function child");
        let mut identifier_references = Vec::new();
        let mut identifier_edges = Vec::new();
        collect_call_symbols(
            identifier_function,
            "",
            Some("run"),
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut identifier_references,
            &mut identifier_edges,
        );
        assert!(
            identifier_references.is_empty() && identifier_edges.is_empty(),
            "mismatched source text should trigger empty-symbol guard and return early"
        );

        let selector_source = r#"
package main
func run() { alias.Do() }
"#;
        let selector_tree = parse_go_root(selector_source);
        let selector_call = find_first_kind(selector_tree.root_node(), "call_expression")
            .expect("selector call expression should exist");
        let selector_function = selector_call
            .child_by_field_name("function")
            .expect("selector call should expose function child");
        let mut selector_references = Vec::new();
        let mut selector_edges = Vec::new();
        collect_call_symbols(
            selector_function,
            "",
            Some("run"),
            "src/main.go",
            "go",
            &HashMap::new(),
            &mut selector_references,
            &mut selector_edges,
        );
        assert!(
            selector_references.is_empty() && selector_edges.is_empty(),
            "mismatched source text should trigger selector field empty-symbol guard"
        );
    }
}
