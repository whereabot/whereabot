//! The agent's reason for a locate request, as the owner reads it on the phone.

/// Longest reason kept, in characters. Long enough for one sentence, short enough for a lock screen.
pub const MAX_CHARS: usize = 200;

/// Trims the agent's text, folds control characters and line breaks into spaces, and caps the length.
/// None when nothing is left.
pub fn clean(raw: Option<&str>) -> Option<String> {
    let folded: String = raw?
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let words = folded.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut reason: String = words.chars().take(MAX_CHARS).collect();
    if words.chars().count() > MAX_CHARS {
        reason.pop();
        reason.push('…');
    }
    (!reason.is_empty()).then_some(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_or_missing_is_none() {
        assert_eq!(clean(None), None);
        assert_eq!(clean(Some(" \n\t ")), None);
    }

    #[test]
    fn folds_line_breaks_and_controls() {
        assert_eq!(
            clean(Some("  Find a\ncafe\u{7} near you ")).as_deref(),
            Some("Find a cafe near you")
        );
    }

    #[test]
    fn caps_long_text_with_an_ellipsis() {
        let reason = clean(Some(&"a".repeat(500))).unwrap();
        assert_eq!(reason.chars().count(), MAX_CHARS);
        assert!(reason.ends_with('…'));
        assert_eq!(
            clean(Some(&"b".repeat(MAX_CHARS))).unwrap(),
            "b".repeat(MAX_CHARS)
        );
    }
}
