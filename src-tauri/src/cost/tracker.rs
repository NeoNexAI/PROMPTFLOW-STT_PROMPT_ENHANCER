//! Per-provider cost estimation.
//!
//! Rates are blended input+output averages per 1K tokens, approximate and
//! updated periodically. They are the single source of truth used by every
//! provider implementation (via `complete`) and by the usage dashboard, so
//! pricing never diverges between the request path and the reporting path.
//! Estimates are surfaced in the UI clearly labelled as "estimated".

/// Estimated cost in USD for `tokens` tokens on `provider`.
pub fn estimate_cost(tokens: u32, provider: &str) -> f64 {
    (tokens as f64 / 1000.0) * rate_per_1k(provider)
}

/// Blended USD cost per 1K tokens for a provider.
fn rate_per_1k(provider: &str) -> f64 {
    match provider {
        "openai" => 0.00015,    // gpt-4o-mini
        "anthropic" => 0.00025, // claude-haiku-4-5
        "mistral" => 0.0001,    // mistral-small
        "gemini" => 0.0,        // free tier
        "groq" => 0.0,          // free tier
        "ollama" => 0.0,        // local inference
        "custom" => 0.0,        // unknown — cost tracking disabled
        "openrouter" => 0.001,  // varies by routed model — conservative estimate
        _ => 0.001,             // conservative default for unknown providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_cost() {
        // 1000 tokens * $0.00015/1k = $0.00015
        assert!((estimate_cost(1000, "openai") - 0.00015).abs() < 1e-12);
    }

    #[test]
    fn test_free_providers_are_zero() {
        for p in ["groq", "gemini", "ollama", "custom"] {
            assert_eq!(estimate_cost(10_000, p), 0.0, "{p} should be free");
        }
    }

    #[test]
    fn test_unknown_provider_uses_conservative_default() {
        assert!((estimate_cost(1000, "mystery") - 0.001).abs() < 1e-12);
    }

    #[test]
    fn test_zero_tokens_is_zero_cost() {
        assert_eq!(estimate_cost(0, "openai"), 0.0);
    }
}
