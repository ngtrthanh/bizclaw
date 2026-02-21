//! JSON grammar constraints for structured output.
//!
//! Constrains token generation to produce valid JSON,
//! enabling reliable tool calling with local models.

/// Grammar state for constraining generation.
#[derive(Debug, Clone)]
pub struct JsonGrammar {
    /// Whether grammar constraint is active.
    pub active: bool,
    /// Current parsing state.
    state: JsonState,
    /// Nesting depth.
    depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum JsonState {
    Start,
    InObject,
    InArray,
    InString,
    InNumber,
    InValue,
    Done,
}

impl JsonGrammar {
    /// Create a new JSON grammar constraint.
    pub fn new() -> Self {
        Self {
            active: false,
            state: JsonState::Start,
            depth: 0,
        }
    }

    /// Enable grammar constraint.
    pub fn enable(&mut self) {
        self.active = true;
        self.state = JsonState::Start;
        self.depth = 0;
    }

    /// Check if a character is valid at the current position.
    pub fn is_valid_char(&self, ch: char) -> bool {
        if !self.active {
            return true;
        }
        match self.state {
            JsonState::Start => ch == '{' || ch == '[' || ch == '"' || ch.is_ascii_digit() || ch == '-' || ch == 't' || ch == 'f' || ch == 'n',
            JsonState::InObject => ch != '\0',
            JsonState::InArray => ch != '\0',
            JsonState::InString => true, // allow anything in strings
            JsonState::InNumber => ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-',
            JsonState::InValue => true,
            JsonState::Done => false,
        }
    }

    /// Check if generation is complete (valid JSON produced).
    pub fn is_complete(&self) -> bool {
        self.state == JsonState::Done || (self.active && self.depth == 0 && self.state != JsonState::Start)
    }

    /// Reset grammar state.
    pub fn reset(&mut self) {
        self.state = JsonState::Start;
        self.depth = 0;
    }
}

impl Default for JsonGrammar {
    fn default() -> Self { Self::new() }
}
