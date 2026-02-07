use crate::indexer::languages::{
    ExtractedEdge, ExtractedReference, ExtractedSymbol, ExtractionUnit, LanguageAdapter, SymbolKey,
};
use crate::indexer::rust_ast;

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
                let to_symbol_key =
                    qualified_module_for_reference(source, reference.line, reference.column)
                        .map(|module_symbol| {
                            qualified_target_symbol_key(
                                file_path,
                                &language,
                                &module_symbol,
                                &reference.symbol,
                            )
                        })
                        .unwrap_or_else(|| SymbolKey {
                            symbol: reference.symbol.clone(),
                            qualified_symbol: None,
                            file_path: Some(file_path.to_string()),
                            language: Some(language.clone()),
                        });
                edges.push(ExtractedEdge {
                    from_symbol_key: scoped_symbol_key(file_path, &language, &caller_symbol),
                    to_symbol_key,
                    edge_kind: "calls".to_string(),
                    confidence: 0.95,
                    provenance: "call_resolution".to_string(),
                });
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

fn qualified_target_symbol_key(
    caller_file_path: &str,
    language: &str,
    module_symbol: &str,
    symbol: &str,
) -> SymbolKey {
    let module_path = module_candidate_path(caller_file_path, module_symbol);
    SymbolKey {
        symbol: symbol.to_string(),
        qualified_symbol: Some(format!("{language}:{module_path}::{symbol}")),
        file_path: Some(module_path),
        language: Some(language.to_string()),
    }
}

fn module_candidate_path(caller_file_path: &str, module_symbol: &str) -> String {
    let parent = std::path::Path::new(caller_file_path)
        .parent()
        .and_then(|path| path.to_str())
        .unwrap_or("");
    if parent.is_empty() {
        format!("{module_symbol}.rs")
    } else {
        format!("{parent}/{module_symbol}.rs")
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
            if let Some((trait_part, type_part)) = rest.split_once(" for ") {
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
    }

    edges
}

fn qualified_module_for_reference(source: &str, line: u32, column: u32) -> Option<String> {
    let line_text = source.lines().nth(line.saturating_sub(1) as usize)?;
    let column_index = column.saturating_sub(1) as usize;
    if column_index > line_text.len() {
        return None;
    }

    let prefix = line_text[..column_index].trim_end();
    let without_separator = prefix.strip_suffix("::")?;
    last_rust_identifier(without_separator)
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
        .filter(|part| !part.is_empty())
        .last()
        .map(str::to_string)
}
