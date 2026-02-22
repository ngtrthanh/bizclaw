//! Grammar-Constrained JSON Decoding.
//!
//! Pre-analyzes vocabulary tokens at load time for JSON structure properties
//! (brace delta, bracket delta, quote parity). During generation, masks logits
//! to guarantee syntactically valid JSON — essential for tool calling with small models.

/// JSON grammar state machine for constrained decoding.
#[derive(Debug, Clone)]
pub struct JsonGrammar {
    /// Pre-computed token properties: (brace_delta, bracket_delta, quote_parity)
    token_props: Vec<TokenJsonProps>,
    /// Current grammar state
    state: JsonState,
}

/// Pre-computed JSON properties per vocabulary token.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokenJsonProps {
    pub brace_delta: i32,   // +1 for {, -1 for }
    pub bracket_delta: i32, // +1 for [, -1 for ]
    pub quote_toggle: bool, // true if contains odd number of unescaped "
    pub is_colon: bool,
    pub is_comma: bool,
    pub is_whitespace_only: bool,
}

/// JSON parsing state — tracks structure validity.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct JsonState {
    pub brace_depth: i32,
    pub bracket_depth: i32,
    pub in_string: bool,
    pub expect_value: bool,
    pub started: bool,
    pub completed: bool,
}


impl JsonGrammar {
    /// Analyze all tokens in vocabulary for JSON properties (done once at load).
    pub fn new(vocab: &[String]) -> Self {
        let token_props: Vec<TokenJsonProps> = vocab
            .iter()
            .map(|token| {
                let mut props = TokenJsonProps::default();
                let mut in_str = false;
                let mut prev_escape = false;

                for ch in token.chars() {
                    if in_str {
                        if ch == '"' && !prev_escape {
                            in_str = false;
                            props.quote_toggle = !props.quote_toggle;
                        }
                        prev_escape = ch == '\\' && !prev_escape;
                    } else {
                        match ch {
                            '{' => props.brace_delta += 1,
                            '}' => props.brace_delta -= 1,
                            '[' => props.bracket_delta += 1,
                            ']' => props.bracket_delta -= 1,
                            '"' => {
                                in_str = true;
                                props.quote_toggle = !props.quote_toggle;
                            }
                            ':' => props.is_colon = true,
                            ',' => props.is_comma = true,
                            _ => {}
                        }
                    }
                }
                props.is_whitespace_only = token.trim().is_empty();
                props
            })
            .collect();

        Self {
            token_props,
            state: JsonState::default(),
        }
    }

    /// Mask logits to only allow tokens that maintain valid JSON.
    pub fn apply_mask(&self, logits: &mut [f32]) {
        if self.state.completed {
            return; // JSON is complete, let EOS through
        }

        for (i, logit) in logits.iter_mut().enumerate() {
            if i >= self.token_props.len() {
                break;
            }
            let props = &self.token_props[i];

            let allowed = self.is_token_allowed(props);
            if !allowed {
                *logit = f32::NEG_INFINITY;
            }
        }
    }

    /// Check if a token is structurally valid given current state.
    fn is_token_allowed(&self, props: &TokenJsonProps) -> bool {
        // In a string: almost anything is allowed
        if self.state.in_string {
            return true;
        }

        // Starting: must begin with { or [
        if !self.state.started {
            return props.brace_delta > 0 || props.bracket_delta > 0;
        }

        // Would close more braces/brackets than open
        let new_brace = self.state.brace_depth + props.brace_delta;
        let new_bracket = self.state.bracket_depth + props.bracket_delta;
        if new_brace < 0 || new_bracket < 0 {
            return false;
        }

        // Whitespace is always fine
        if props.is_whitespace_only {
            return true;
        }

        true
    }

    /// Update state after a token is selected.
    pub fn accept_token(&mut self, token_id: usize) {
        if token_id >= self.token_props.len() {
            return;
        }
        let props = &self.token_props[token_id];

        self.state.brace_depth += props.brace_delta;
        self.state.bracket_depth += props.bracket_delta;

        if props.quote_toggle {
            self.state.in_string = !self.state.in_string;
        }

        if !self.state.started && (props.brace_delta > 0 || props.bracket_delta > 0) {
            self.state.started = true;
        }

        if self.state.started
            && self.state.brace_depth == 0
            && self.state.bracket_depth == 0
            && !self.state.in_string
        {
            self.state.completed = true;
        }
    }

    /// Check if JSON generation is complete.
    pub fn is_complete(&self) -> bool {
        self.state.completed
    }

    /// Reset grammar state for new generation.
    pub fn reset(&mut self) {
        self.state = JsonState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_token_analysis() {
        let vocab = vec![
            "{".to_string(),
            "}".to_string(),
            "[".to_string(),
            "]".to_string(),
            "\"hello\"".to_string(),
            ":".to_string(),
            ",".to_string(),
            " ".to_string(),
        ];
        let grammar = JsonGrammar::new(&vocab);
        assert_eq!(grammar.token_props[0].brace_delta, 1);
        assert_eq!(grammar.token_props[1].brace_delta, -1);
        assert_eq!(grammar.token_props[2].bracket_delta, 1);
        assert!(grammar.token_props[5].is_colon);
        assert!(grammar.token_props[7].is_whitespace_only);
    }

    #[test]
    fn test_grammar_completion() {
        let vocab = vec![
            "{".to_string(),
            "}".to_string(),
            "\"key\"".to_string(),
            ":".to_string(),
            "\"val\"".to_string(),
        ];
        let mut grammar = JsonGrammar::new(&vocab);

        grammar.accept_token(0); // {
        assert!(!grammar.is_complete());

        grammar.accept_token(2); // "key"
        grammar.accept_token(3); // :
        grammar.accept_token(4); // "val"

        grammar.accept_token(1); // }
        assert!(grammar.is_complete());
    }

    #[test]
    fn test_grammar_mask_initial() {
        let vocab = vec![
            "{".to_string(),
            "}".to_string(),
            "hello".to_string(),
            "[".to_string(),
        ];
        let grammar = JsonGrammar::new(&vocab);

        let mut logits = vec![1.0, 1.0, 1.0, 1.0];
        grammar.apply_mask(&mut logits);

        // Only { and [ should be allowed at start
        assert!(logits[0].is_finite()); // {
        assert!(logits[1] == f32::NEG_INFINITY); // }
        assert!(logits[2] == f32::NEG_INFINITY); // hello
        assert!(logits[3].is_finite()); // [
    }
}
