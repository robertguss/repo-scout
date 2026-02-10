use crate::indexer::languages::{
    ExtractedEdge, ExtractedReference, ExtractedSymbol, ExtractionUnit, LanguageAdapter, SymbolKey,
};
use crate::indexer::rust_ast;
use std::collections::HashSet;

pub struct RustLanguageAdapter;

impl LanguageAdapter for RustLanguageAdapter {
    fn language_id(&self) -> &'static str {
        "rust"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn extract(&self, file_path: &str, source: &str) -> anyhow::Result<ExtractionUnit> {
        let (definitions, references) = rust_ast::extract_rust_items(source)?;
        let language = self.language_id().to_string();

        let mut symbols = Vec::new();
        let mut references_out = Vec::new();
        let mut edges = Vec::new();

        for definition in definitions {
            let symbol = definition.symbol;
            let kind = definition.kind;
            let container = definition.container;
            let extracted_symbol = ExtractedSymbol {
                qualified_symbol: Some(format!("rust:{file_path}::{symbol}")),
                language: language.clone(),
                symbol: symbol.clone(),
                kind,
                container: container.clone(),
                start_line: definition.line,
                start_column: definition.column,
                end_line: definition.end_line,
                end_column: definition.end_column,
                signature: definition.signature,
            };
            symbols.push(extracted_symbol);

            if let Some(container_symbol) = container {
                edges.push(ExtractedEdge {
                    from_symbol_key: scoped_symbol_key(file_path, &language, &container_symbol),
                    to_symbol_key: scoped_symbol_key(file_path, &language, &symbol),
                    edge_kind: "contains".to_string(),
                    confidence: 1.0,
                    provenance: "ast_definition".to_string(),
                });
            }
        }

        for reference in references {
            references_out.push(ExtractedReference {
                symbol: reference.symbol.clone(),
                line: reference.line,
                column: reference.column,
            });

            if let Some(caller_symbol) = reference.caller {
                let from_symbol_key = scoped_symbol_key(file_path, &language, &caller_symbol);
                for to_symbol_key in qualified_target_symbol_keys(
                    file_path,
                    &language,
                    source,
                    reference.line,
                    reference.column,
                    &reference.symbol,
                ) {
                    edges.push(ExtractedEdge {
                        from_symbol_key: from_symbol_key.clone(),
                        to_symbol_key,
                        edge_kind: "calls".to_string(),
                        confidence: 0.95,
                        provenance: "call_resolution".to_string(),
                    });
                }
            }
        }

        edges.extend(relation_hints(file_path, source, &language));

        Ok(ExtractionUnit {
            symbols,
            references: references_out,
            edges,
        })
    }
}

fn scoped_symbol_key(file_path: &str, language: &str, symbol: &str) -> SymbolKey {
    SymbolKey {
        symbol: symbol.to_string(),
        qualified_symbol: Some(format!("{language}:{file_path}::{symbol}")),
        file_path: Some(file_path.to_string()),
        language: Some(language.to_string()),
    }
}

fn qualified_target_symbol_keys(
    caller_file_path: &str,
    language: &str,
    source: &str,
    line: u32,
    column: u32,
    symbol: &str,
) -> Vec<SymbolKey> {
    let mut keys = Vec::new();
    if let Some(module_segments) = qualified_module_segments_for_reference(source, line, column) {
        for module_path in qualified_module_candidate_paths(caller_file_path, &module_segments) {
            keys.push(SymbolKey {
                symbol: symbol.to_string(),
                qualified_symbol: Some(format!("{language}:{module_path}::{symbol}")),
                file_path: Some(module_path),
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
    dedupe_symbol_keys(keys)
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

fn qualified_module_candidate_paths(caller_file_path: &str, segments: &[String]) -> Vec<String> {
    let Some(resolved_segments) = resolve_module_segments_for_reference(caller_file_path, segments)
    else {
        return Vec::new();
    };

    let crate_root = crate_root_prefix(caller_file_path);
    let mut candidates = Vec::new();
    if resolved_segments.is_empty() {
        candidates.push(join_module_candidate_path(crate_root, "lib.rs"));
        candidates.push(join_module_candidate_path(crate_root, "main.rs"));
    } else {
        let module_rel = resolved_segments.join("/");
        candidates.push(join_module_candidate_path(
            crate_root,
            &format!("{module_rel}.rs"),
        ));
        candidates.push(join_module_candidate_path(
            crate_root,
            &format!("{module_rel}/mod.rs"),
        ));
    }

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for candidate in candidates {
        if seen.insert(candidate.clone()) {
            deduped.push(candidate);
        }
    }
    deduped
}

fn resolve_module_segments_for_reference(
    caller_file_path: &str,
    segments: &[String],
) -> Option<Vec<String>> {
    if segments.is_empty() {
        return None;
    }

    let mut resolved = current_module_segments(caller_file_path);
    let mut index = 0usize;
    match segments.first().map(String::as_str) {
        Some("crate") => {
            resolved.clear();
            index = 1;
        }
        Some("self") => {
            index = 1;
        }
        Some("super") => {
            while index < segments.len() && segments[index] == "super" {
                resolved.pop()?;
                index += 1;
            }
        }
        _ => {}
    }

    for segment in &segments[index..] {
        resolved.push(segment.clone());
    }

    Some(resolved)
}

fn current_module_segments(caller_file_path: &str) -> Vec<String> {
    let path_parts = caller_file_path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if path_parts.is_empty() {
        return Vec::new();
    }

    let file_name = path_parts[path_parts.len() - 1];
    let path_start = usize::from(path_parts.len() > 1);
    let mut segments = path_parts[path_start..path_parts.len().saturating_sub(1)]
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();

    if !matches!(file_name, "lib.rs" | "main.rs" | "mod.rs")
        && let Some(stem) = file_name.strip_suffix(".rs")
        && !stem.is_empty()
    {
        segments.push(stem.to_string());
    }

    segments
}

fn crate_root_prefix(caller_file_path: &str) -> &str {
    let mut components = caller_file_path
        .split('/')
        .filter(|component| !component.is_empty());
    let Some(first) = components.next() else {
        return "";
    };
    if components.next().is_some() {
        first
    } else {
        ""
    }
}

fn join_module_candidate_path(crate_root: &str, relative: &str) -> String {
    if crate_root.is_empty() {
        relative.to_string()
    } else {
        format!("{crate_root}/{relative}")
    }
}

fn relation_hints(file_path: &str, content: &str, language: &str) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("use ") {
            let statement = rest.trim().trim_end_matches(';').trim();
            if let Some((left, right)) = statement.split_once(" as ") {
                let Some(target_symbol) = last_rust_identifier(left) else {
                    continue;
                };
                let Some(alias_symbol) = last_rust_identifier(right) else {
                    continue;
                };
                if alias_symbol != target_symbol {
                    edges.push(ExtractedEdge {
                        from_symbol_key: scoped_symbol_key(file_path, language, &alias_symbol),
                        to_symbol_key: scoped_symbol_key(file_path, language, &target_symbol),
                        edge_kind: "imports".to_string(),
                        confidence: 0.9,
                        provenance: "import_resolution".to_string(),
                    });
                }
            }
        }

        if let Some(rest) = trimmed.strip_prefix("impl") {
            let rest = rest.trim_start();
            let Some(rest) = strip_leading_impl_generics(rest) else {
                continue;
            };
            let rest = rest.trim_start();
            let Some((trait_part, type_part)) = rest.split_once(" for ") else {
                continue;
            };
            let trait_head = trait_part.split('<').next().unwrap_or(trait_part);
            let type_head = type_part.split('<').next().unwrap_or(type_part);
            let Some(trait_symbol) = last_rust_identifier(trait_head) else {
                continue;
            };
            let Some(type_symbol) = last_rust_identifier(type_head) else {
                continue;
            };
            edges.push(ExtractedEdge {
                from_symbol_key: scoped_symbol_key(file_path, language, &type_symbol),
                to_symbol_key: scoped_symbol_key(file_path, language, &trait_symbol),
                edge_kind: "implements".to_string(),
                confidence: 0.95,
                provenance: "ast_reference".to_string(),
            });
        }
    }

    edges
}

fn qualified_module_segments_for_reference(
    source: &str,
    line: u32,
    column: u32,
) -> Option<Vec<String>> {
    let line_text = source.lines().nth(line.saturating_sub(1) as usize)?;
    let column_index = column.saturating_sub(1) as usize;
    if column_index > line_text.len() {
        return None;
    }

    let prefix = line_text[..column_index].trim_end();
    let without_separator = prefix.strip_suffix("::")?;
    path_suffix_segments(without_separator)
}

fn path_suffix_segments(segment: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut end = segment.len();

    loop {
        let start = identifier_start_index(segment, end)?;
        parts.push(segment[start..end].to_string());

        if start < 2 || &segment[start - 2..start] != "::" {
            break;
        }
        end = start - 2;
    }

    parts.reverse();
    Some(parts)
}

fn identifier_start_index(segment: &str, end: usize) -> Option<usize> {
    let bytes = segment.as_bytes();
    let mut start = end;
    while start > 0 {
        let byte = bytes[start - 1];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            start -= 1;
            continue;
        }
        break;
    }
    (start != end).then_some(start)
}

fn strip_leading_impl_generics(segment: &str) -> Option<&str> {
    if !segment.starts_with('<') {
        return Some(segment);
    }

    let mut depth = 0i32;
    for (index, ch) in segment.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&segment[index + 1..]);
                }
            }
            _ => {}
        }
    }
    None
}

fn last_rust_identifier(segment: &str) -> Option<String> {
    segment
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .rfind(|part| !part.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_extract_emits_call_edges_with_caller_context() {
        let source = r#"
fn callee() {}

fn caller() {
    callee();
}

callee();
"#;
        let unit = RustLanguageAdapter
            .extract("src/lib.rs", source)
            .expect("rust extraction should succeed");
        assert!(
            unit.references.iter().any(|item| item.symbol == "callee"),
            "call references should be captured"
        );
        assert!(
            unit.edges.iter().any(|edge| {
                edge.edge_kind == "calls"
                    && edge.from_symbol_key.symbol == "caller"
                    && edge.to_symbol_key.symbol == "callee"
            }),
            "caller-scoped call edges should be emitted"
        );
    }

    #[test]
    fn module_resolution_helpers_cover_empty_and_unbalanced_paths() {
        assert!(
            qualified_module_candidate_paths("src/lib.rs", &[]).is_empty(),
            "empty reference segments should not produce module candidates"
        );
        assert_eq!(
            resolve_module_segments_for_reference("src/lib.rs", &[]),
            None
        );
        assert!(
            current_module_segments("").is_empty(),
            "empty caller paths should produce no module segments"
        );
        assert_eq!(crate_root_prefix(""), "");
        assert_eq!(crate_root_prefix("main.rs"), "");
        assert_eq!(
            join_module_candidate_path("", "lib.rs"),
            "lib.rs".to_string()
        );
        assert_eq!(path_suffix_segments("::"), None);
        assert_eq!(
            path_suffix_segments("crate::pkg::Type"),
            Some(vec![
                "crate".to_string(),
                "pkg".to_string(),
                "Type".to_string()
            ])
        );
        assert_eq!(
            qualified_module_segments_for_reference("foo();", 1, 99),
            None
        );
        assert_eq!(strip_leading_impl_generics("<T"), None);
        assert_eq!(strip_leading_impl_generics("<T>Type"), Some("Type"));
    }

    #[test]
    fn relation_hints_cover_invalid_and_valid_use_impl_forms() {
        let source = r#"
use :: as alias;
use crate::foo as ;
impl <T for Missing {}
impl Missing {}
impl :: for MyType {}
impl Trait for :: {}
impl Trait for MyType {}
"#;
        let edges = relation_hints("src/lib.rs", source, "rust");
        assert!(
            edges.iter().any(|edge| {
                edge.edge_kind == "implements"
                    && edge.from_symbol_key.symbol == "MyType"
                    && edge.to_symbol_key.symbol == "Trait"
            }),
            "valid impl lines should still produce implements edges"
        );
        assert_eq!(
            edges
                .iter()
                .filter(|edge| edge.edge_kind == "implements")
                .count(),
            1,
            "malformed impl/use lines should be ignored"
        );
    }

    #[test]
    fn relation_hints_cover_missing_alias_identifier_and_balanced_impl_generics() {
        let source = r#"
use crate::foo as ::;
impl<T> Trait for MyType {}
"#;
        let edges = relation_hints("src/lib.rs", source, "rust");
        assert!(
            edges.iter().any(|edge| {
                edge.edge_kind == "implements"
                    && edge.from_symbol_key.symbol == "MyType"
                    && edge.to_symbol_key.symbol == "Trait"
            }),
            "balanced impl generics should still emit implements edges"
        );
        assert_eq!(
            strip_leading_impl_generics("<T>MyType"),
            Some("MyType"),
            "balanced generic parameter lists should strip the leading impl generic clause"
        );
    }

    #[test]
    fn strip_leading_impl_generics_supports_nested_generic_bounds() {
        assert_eq!(
            strip_leading_impl_generics("<T: Trait<U>>MyType"),
            Some("MyType"),
            "nested generic bounds should still strip balanced impl generics"
        );
    }
}
