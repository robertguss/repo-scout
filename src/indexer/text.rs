#[derive(Debug, Clone)]
pub struct TokenOccurrence {
    pub symbol: String,
    pub line: u32,
    pub column: u32,
}

pub fn extract_token_occurrences(content: &str) -> Vec<TokenOccurrence> {
    let mut occurrences = Vec::new();

    for (line_index, line_text) in content.lines().enumerate() {
        let line_number = (line_index + 1) as u32;
        let mut token_start: Option<usize> = None;

        for (byte_index, ch) in line_text.char_indices() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                if token_start.is_none() {
                    token_start = Some(byte_index);
                }
                continue;
            }

            if let Some(start) = token_start.take() {
                push_token(&mut occurrences, line_text, start, byte_index, line_number);
            }
        }

        if let Some(start) = token_start {
            push_token(
                &mut occurrences,
                line_text,
                start,
                line_text.len(),
                line_number,
            );
        }
    }

    occurrences
}

fn push_token(
    occurrences: &mut Vec<TokenOccurrence>,
    line_text: &str,
    start: usize,
    end: usize,
    line_number: u32,
) {
    let token = &line_text[start..end];
    if token.is_empty() {
        return;
    }

    let first = token.as_bytes()[0] as char;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return;
    }

    occurrences.push(TokenOccurrence {
        symbol: token.to_string(),
        line: line_number,
        column: (start + 1) as u32,
    });
}
