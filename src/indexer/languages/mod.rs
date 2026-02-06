pub mod rust;

#[derive(Debug, Clone)]
pub struct SymbolKey {
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub symbol: String,
    pub qualified_symbol: Option<String>,
    pub kind: String,
    pub language: String,
    pub container: Option<String>,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtractedEdge {
    pub from_symbol_key: SymbolKey,
    pub to_symbol_key: SymbolKey,
    pub edge_kind: String,
    pub confidence: f64,
    pub provenance: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedReference {
    pub symbol: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractionUnit {
    pub symbols: Vec<ExtractedSymbol>,
    pub references: Vec<ExtractedReference>,
    pub edges: Vec<ExtractedEdge>,
}

pub trait LanguageAdapter {
    fn language_id(&self) -> &'static str;
    fn file_extensions(&self) -> &'static [&'static str];
    fn extract(&self, file_path: &str, source: &str) -> anyhow::Result<ExtractionUnit>;
}
