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
            if let Some(symbol) = node_text(node, source) {
                let (line, column) = start_position(node);
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
                        None,
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
            }
        }
        "selector_expression" => {
            let qualifier = node
                .child_by_field_name("operand")
                .and_then(|operand| node_text(operand, source))
                .and_then(|text| first_identifier(&text));
            if let Some(field_node) = node.child_by_field_name("field") {
                if let Some(symbol) = node_text(field_node, source) {
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
                                from_symbol_key: scoped_symbol_key(
                                    file_path,
                                    language,
                                    caller_symbol,
                                ),
                                to_symbol_key,
                                edge_kind: "calls".to_string(),
                                confidence: 0.95,
                                provenance: "call_resolution".to_string(),
                            });
                        }
                    }
                }
                return;
            }
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
