//! Small production primitives checked by the independent Kani pilot.
//! Keep dependencies out: verification imports this exact file, not a model.

/// Return the next budget total only when addition is representable and fits.
pub fn reserve_budget(used: usize, cost: usize, budget: usize) -> Option<usize> {
    used.checked_add(cost).filter(|next| *next <= budget)
}

/// In strict mode, only a complete analysis with no breaking changes passes.
pub fn contract_gate(strict: bool, complete: bool, breaking: usize) -> bool {
    !strict || (complete && breaking == 0)
}

/// Replace a UTF-8 byte range, preserving all bytes outside that range.
/// Marker discovery, template rendering and filesystem writes are separate.
pub fn replace_range(existing: &str, start: usize, end: usize, block: &str) -> Option<String> {
    if start > end || !existing.is_char_boundary(start) || !existing.is_char_boundary(end) {
        return None;
    }
    let capacity = (existing.len() - (end - start)).checked_add(block.len())?;
    let mut result = String::with_capacity(capacity);
    result.push_str(&existing[..start]);
    result.push_str(block);
    result.push_str(&existing[end..]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unicode_crlf_and_rejects_invalid_ranges() {
        assert_eq!(
            replace_range("é\r\n旧\r\n🦀", 4, 7, "novo").as_deref(),
            Some("é\r\nnovo\r\n🦀")
        );
        assert_eq!(replace_range("é", 1, 2, ""), None);
        assert_eq!(replace_range("abc", 2, 1, ""), None);
        assert_eq!(replace_range("abc", 0, usize::MAX, ""), None);
    }

    #[test]
    fn range_replacement_matches_independent_unicode_splice() {
        for source in ["", "a", "é", "a旧🦀\r\nz", "prefix <!-- marker --> suffix"] {
            for block in ["", "x", "é🦀", "\r\n"] {
                for start in 0..=source.len() + 1 {
                    for end in 0..=source.len() + 1 {
                        let actual = replace_range(source, start, end, block);
                        if start <= end
                            && source.is_char_boundary(start)
                            && source.is_char_boundary(end)
                        {
                            let mut expected = source.to_owned();
                            expected.replace_range(start..end, block);
                            assert_eq!(actual.as_deref(), Some(expected.as_str()));
                        } else {
                            assert_eq!(actual, None);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn rejects_overflow_and_incomplete_analysis() {
        assert_eq!(reserve_budget(usize::MAX, 1, usize::MAX), None);
        assert_eq!(reserve_budget(2, 3, 5), Some(5));
        assert!(!contract_gate(true, false, 0));
        assert!(contract_gate(false, false, 1));
    }
}
