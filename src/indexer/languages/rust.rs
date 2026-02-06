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

        for reference in references {
            references_out.push(ExtractedReference {
                symbol: reference.symbol.clone(),
                line: reference.line,
                column: reference.column,
            });

            if let Some(caller_symbol) = reference.caller {
                edges.push(ExtractedEdge {
                    from_symbol_key: SymbolKey {
                        symbol: caller_symbol,
                    },
                    to_symbol_key: SymbolKey {
                        symbol: reference.symbol,
                    },
                    edge_kind: "calls".to_string(),
                    confidence: 0.95,
                    provenance: "call_resolution".to_string(),
                });
            }
        }

        edges.extend(relation_hints(source).into_iter().map(
            |(from_symbol, to_symbol, edge_kind, confidence, provenance)| ExtractedEdge {
                from_symbol_key: SymbolKey {
                    symbol: from_symbol,
                },
                to_symbol_key: SymbolKey { symbol: to_symbol },
                edge_kind,
                confidence,
                provenance,
            },
        ));

        Ok(ExtractionUnit {
            symbols,
            references: references_out,
            edges,
        })
    }
}

fn relation_hints(content: &str) -> Vec<(String, String, String, f64, String)> {
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
                    edges.push((
                        alias_symbol,
                        target_symbol,
                        "imports".to_string(),
                        0.9,
                        "import_resolution".to_string(),
                    ));
                }
            }
        }

        if let Some(rest) = trimmed.strip_prefix("impl ")
            && let Some((trait_part, type_part)) = rest.split_once(" for ")
        {
            let Some(trait_symbol) = last_rust_identifier(trait_part) else {
                continue;
            };
            let Some(type_symbol) = last_rust_identifier(type_part) else {
                continue;
            };
            edges.push((
                type_symbol,
                trait_symbol,
                "implements".to_string(),
                0.95,
                "ast_reference".to_string(),
            ));
        }
    }

    edges
}

fn last_rust_identifier(segment: &str) -> Option<String> {
    segment
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty())
        .last()
        .map(str::to_string)
}
