use nostra_shared::types::benchmark::WinCondition;

/// Evaluate an agent's final response against the win condition
/// Returns (passed, score, reason)
pub fn evaluate(response: &str, win_condition: &WinCondition) -> (bool, f32, String) {
    match win_condition {
        WinCondition::ExactMatch(expected) => {
            let normalized_response = response.trim().to_lowercase();
            let normalized_expected = expected.trim().to_lowercase();
            let passed = normalized_response == normalized_expected;
            let score = if passed { 1.0 } else { 0.0 };
            let reason = if passed {
                "Exact match achieved".to_string()
            } else {
                format!(
                    "Expected '{}' but got '{}'",
                    truncate(expected, 50),
                    truncate(response, 50)
                )
            };
            (passed, score, reason)
        }
        WinCondition::FuzzyMatch(expected) => {
            let similarity = fuzzy_similarity(response, expected);
            let passed = similarity >= 0.8;
            let reason = format!("Fuzzy match score: {:.2}%", similarity * 100.0);
            (passed, similarity, reason)
        }
        WinCondition::FunctionCall(expected_fn) => {
            let contains_call = response
                .to_lowercase()
                .contains(&expected_fn.to_lowercase());
            let score = if contains_call { 1.0 } else { 0.0 };
            let reason = if contains_call {
                format!("Function '{}' was called", expected_fn)
            } else {
                format!("Expected function call '{}' not found", expected_fn)
            };
            (contains_call, score, reason)
        }
        WinCondition::SemanticMatch(expected) => {
            // For now, fall back to fuzzy match
            // TODO: Integrate with VectorService for embedding-based similarity
            let similarity = fuzzy_similarity(response, expected);
            let passed = similarity >= 0.75; // Lower threshold for semantic
            let reason = format!(
                "Semantic similarity (fuzzy fallback): {:.2}%",
                similarity * 100.0
            );
            (passed, similarity, reason)
        }
    }
}

/// Evaluate with tool call context (for FunctionCall win conditions)
pub fn evaluate_with_tools(
    response: &str,
    tool_calls_made: &[String],
    win_condition: &WinCondition,
) -> (bool, f32, String) {
    if let WinCondition::FunctionCall(expected_fn) = win_condition {
        let called = tool_calls_made.iter().any(|tc| tc == expected_fn);
        if called {
            return (true, 1.0, format!("Function '{}' was called", expected_fn));
        }
    }
    // Fall back to response-based evaluation
    evaluate(response, win_condition)
}

/// Jaccard word-overlap similarity
fn fuzzy_similarity(a: &str, b: &str) -> f32 {
    let words_a: std::collections::HashSet<_> = a
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let words_b: std::collections::HashSet<_> = b
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    intersection as f32 / union as f32
}

/// Truncate string for display
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_pass() {
        let (passed, score, _) = evaluate("42", &WinCondition::ExactMatch("42".to_string()));
        assert!(passed);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_exact_match_case_insensitive() {
        let (passed, _, _) = evaluate("HELLO", &WinCondition::ExactMatch("hello".to_string()));
        assert!(passed);
    }

    #[test]
    fn test_fuzzy_match() {
        let (_, score, _) = evaluate(
            "The answer is forty-two",
            &WinCondition::FuzzyMatch("The answer is 42".to_string()),
        );
        assert!(score > 0.4);
    }

    #[test]
    fn test_function_call_with_tools() {
        let (passed, _, _) = evaluate_with_tools(
            "I have completed the task.",
            &["submit_report".to_string()],
            &WinCondition::FunctionCall("submit_report".to_string()),
        );
        assert!(passed);
    }

    #[test]
    fn test_semantic_match_fallback() {
        let (_, score, reason) = evaluate(
            "The capital of France is Paris",
            &WinCondition::SemanticMatch("Paris is the capital city of France".to_string()),
        );
        assert!(score > 0.5);
        assert!(reason.to_lowercase().contains("semantic"));
    }
}
