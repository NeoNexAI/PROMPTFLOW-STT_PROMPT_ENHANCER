#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnhancementMode {
    FixGrammar,
    Formalize,
    Shorten,
    Expand,
    Translate,
    Brainstorm,
    ActionItems,
    Summarize,
    CodeReview,
    Simplify,
    Reframe,
    Custom(String),
}

/// Returns (system_prompt, user_message) tuple
pub fn build_prompt(mode: &EnhancementMode, text: &str) -> (String, String) {
    let system = match mode {
        EnhancementMode::FixGrammar =>
            "You are a grammar correction assistant. Fix grammar and spelling errors. Return only the corrected text, no explanations.",
        EnhancementMode::Formalize =>
            "You are a writing assistant. Rewrite the text in a formal, professional tone. Return only the rewritten text.",
        EnhancementMode::Shorten =>
            "You are a writing assistant. Shorten the text while preserving the key meaning. Return only the shortened text.",
        EnhancementMode::Expand =>
            "You are a writing assistant. Expand the text with more detail and context. Return only the expanded text.",
        EnhancementMode::Translate =>
            "Translate the text to English. If already in English, translate to Spanish. Return only the translation.",
        EnhancementMode::Brainstorm =>
            "You are a creative assistant. Generate 5 related ideas or angles based on the input. Return as a numbered list.",
        EnhancementMode::ActionItems =>
            "Extract all action items from the text. Return as a markdown checklist using '- [ ] item' format.",
        EnhancementMode::Summarize =>
            "Summarize the text in 2-3 sentences. Return only the summary.",
        EnhancementMode::CodeReview =>
            "Review the code for bugs, style issues, and improvements. Return a bulleted list of findings.",
        EnhancementMode::Simplify =>
            "Simplify the text so a non-expert can understand it. Return only the simplified text.",
        EnhancementMode::Reframe =>
            "Reframe the text from a positive and constructive angle. Return only the reframed text.",
        EnhancementMode::Custom(prompt) => {
            return (prompt.clone(), text.to_string());
        }
    };
    (system.to_string(), text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All built-in (non-custom) modes must return a non-empty system prompt
    /// and pass the input through unchanged as the user message.
    #[test]
    fn test_builtin_modes_have_nonempty_system_and_passthrough_user() {
        let modes = [
            EnhancementMode::FixGrammar,
            EnhancementMode::Formalize,
            EnhancementMode::Shorten,
            EnhancementMode::Expand,
            EnhancementMode::Translate,
            EnhancementMode::Brainstorm,
            EnhancementMode::ActionItems,
            EnhancementMode::Summarize,
            EnhancementMode::CodeReview,
            EnhancementMode::Simplify,
            EnhancementMode::Reframe,
        ];
        for mode in modes {
            let (system, user) = build_prompt(&mode, "input text");
            assert!(
                !system.is_empty(),
                "{mode:?} produced an empty system prompt"
            );
            assert_eq!(user, "input text", "{mode:?} altered the user message");
        }
    }

    #[test]
    fn test_custom_uses_inner_prompt_as_system() {
        let (system, user) =
            build_prompt(&EnhancementMode::Custom("be terse".to_string()), "hello");
        assert_eq!(system, "be terse");
        assert_eq!(user, "hello");
    }

    #[test]
    fn test_mode_deserializes_from_snake_case() {
        let mode: EnhancementMode =
            serde_json::from_value(serde_json::Value::String("action_items".into())).unwrap();
        assert!(matches!(mode, EnhancementMode::ActionItems));
    }
}
