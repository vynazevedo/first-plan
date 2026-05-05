//! Identifier-aware tokenization.
//!
//! Converts code identifiers into searchable tokens by splitting:
//! - snake_case  -> snake, case
//! - camelCase   -> camel, case
//! - PascalCase  -> pascal, case
//! - UPPER_CASE  -> upper, case
//! - Numbers separated from words: getUser2 -> get, user, 2
//!
//! All tokens are lowercased. Tokens shorter than 2 chars are dropped (except
//! single-digit numbers, which are kept). Common stop words for code are
//! filtered.

use std::collections::HashSet;

/// Returns true for tokens that add little signal (very common code words).
fn is_stop_word(token: &str) -> bool {
    matches!(
        token,
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "this"
            | "that"
            | "get"
            | "set"
            | "is"
            | "of"
            | "to"
            | "in"
            | "on"
            | "by"
            | "as"
            | "an"
            | "a"
    )
}

/// Tokenize a single identifier or text snippet into search tokens.
///
/// Splits on:
/// - non-alphanumeric (`my-func` -> `my`, `func`)
/// - case boundaries (`getUser` -> `get`, `user`)
/// - letter/number boundaries (`http2` -> `http`, `2`)
///
/// Drops tokens shorter than 2 chars unless purely numeric.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut prev_kind = CharKind::Other;

    for ch in text.chars() {
        let kind = classify(ch);
        let boundary = match (prev_kind, kind) {
            (CharKind::Lower, CharKind::Upper) => true, // camelCase -> camel/Case
            (CharKind::Lower, CharKind::Digit) => true, // foo2 -> foo/2
            (CharKind::Upper, CharKind::Digit) => true,
            (CharKind::Letter, CharKind::Digit) => true,
            (CharKind::Digit, CharKind::Lower) => true,
            (CharKind::Digit, CharKind::Upper) => true,
            (CharKind::Digit, CharKind::Letter) => true,
            (_, CharKind::Other) => true,
            (CharKind::Other, _) => true,
            _ => false,
        };

        if boundary && !current.is_empty() {
            push_token(&mut tokens, &current);
            current.clear();
        }

        if !matches!(kind, CharKind::Other) {
            current.push(ch);
        }

        prev_kind = kind;
    }

    if !current.is_empty() {
        push_token(&mut tokens, &current);
    }

    tokens
}

fn push_token(tokens: &mut Vec<String>, token: &str) {
    let lower = token.to_lowercase();
    if lower.len() < 2 && !lower.chars().all(|c| c.is_ascii_digit()) {
        return;
    }
    if is_stop_word(&lower) {
        return;
    }
    tokens.push(lower);
}

#[derive(Debug, Clone, Copy)]
enum CharKind {
    Upper,
    Lower,
    Letter, // for non-ascii letters
    Digit,
    Other,
}

fn classify(ch: char) -> CharKind {
    if ch.is_ascii_uppercase() {
        CharKind::Upper
    } else if ch.is_ascii_lowercase() {
        CharKind::Lower
    } else if ch.is_alphabetic() {
        CharKind::Letter
    } else if ch.is_ascii_digit() {
        CharKind::Digit
    } else {
        CharKind::Other
    }
}

/// Tokenize and return a `HashSet` for set operations.
pub fn tokenize_set(text: &str) -> HashSet<String> {
    tokenize(text).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case() {
        assert_eq!(
            tokenize("validate_email_format"),
            vec!["validate", "email", "format"]
        );
    }

    #[test]
    fn camel_case() {
        assert_eq!(
            tokenize("validateEmailFormat"),
            vec!["validate", "email", "format"]
        );
    }

    #[test]
    fn pascal_case() {
        assert_eq!(
            tokenize("ValidateEmailFormat"),
            vec!["validate", "email", "format"]
        );
    }

    #[test]
    fn upper_case() {
        assert_eq!(tokenize("MAX_RETRIES"), vec!["max", "retries"]);
    }

    #[test]
    fn drops_short_tokens() {
        assert_eq!(tokenize("a-b-c-foo"), vec!["foo"]);
    }

    #[test]
    fn keeps_single_digits() {
        assert_eq!(tokenize("http2"), vec!["http", "2"]);
    }

    #[test]
    fn natural_language() {
        let tokens = tokenize("Validates an email address against RFC 5322");
        assert!(tokens.contains(&"validates".to_string()));
        assert!(tokens.contains(&"email".to_string()));
        assert!(tokens.contains(&"address".to_string()));
        assert!(tokens.contains(&"rfc".to_string()));
        assert!(tokens.contains(&"5322".to_string()));
        assert!(!tokens.contains(&"the".to_string())); // stop word
        assert!(!tokens.contains(&"an".to_string())); // stop word
    }

    #[test]
    fn drops_stop_words() {
        assert_eq!(tokenize("get_the_user"), vec!["user"]);
    }
}
