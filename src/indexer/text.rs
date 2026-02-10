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

#[cfg(test)]
mod tests {
    use super::{TokenOccurrence, extract_token_occurrences, push_token};

    #[test]
    fn extract_token_occurrences_collects_ascii_identifiers() {
        let occurrences = extract_token_occurrences("alpha beta_2\n3gamma _delta");
        let symbols = occurrences
            .iter()
            .map(|item| item.symbol.as_str())
            .collect::<Vec<_>>();
        assert_eq!(symbols, vec!["alpha", "beta_2", "_delta"]);
    }

    #[test]
    fn push_token_ignores_empty_and_non_identifier_tokens() {
        let mut occurrences = Vec::<TokenOccurrence>::new();
        push_token(&mut occurrences, "abc", 1, 1, 1);
        push_token(&mut occurrences, "1abc", 0, 4, 1);
        push_token(&mut occurrences, "_ok", 0, 3, 2);
        assert_eq!(occurrences.len(), 1);
        assert_eq!(occurrences[0].symbol, "_ok");
        assert_eq!(occurrences[0].line, 2);
    }
}
