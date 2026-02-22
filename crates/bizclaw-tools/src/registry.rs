//! Tool registry — dynamic tool discovery and execution.

use bizclaw_core::traits::Tool;
use bizclaw_core::types::ToolDefinition;

/// Find a tool by name from a list.
pub fn find_tool<'a>(tools: &'a [Box<dyn Tool>], name: &str) -> Option<&'a dyn Tool> {
    tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
}

/// Get all tool definitions from a list.
pub fn list_definitions(tools: &[Box<dyn Tool>]) -> Vec<ToolDefinition> {
    tools.iter().map(|t| t.definition()).collect()
}

/// Validate that a tool call has the required arguments.
pub fn validate_args(definition: &ToolDefinition, args: &serde_json::Value) -> Result<(), String> {
    let params = &definition.parameters;
    if let Some(required) = params.get("required").and_then(|r| r.as_array()) {
        for req in required {
            if let Some(key) = req.as_str()
                && args.get(key).is_none() {
                    return Err(format!("Missing required argument: {key}"));
                }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args_missing() {
        let def = ToolDefinition {
            name: "test".into(),
            description: "test tool".into(),
            parameters: serde_json::json!({
                "required": ["cmd"],
                "properties": {
                    "cmd": { "type": "string" }
                }
            }),
        };

        // Missing required arg
        let result = validate_args(&def, &serde_json::json!({}));
        assert!(result.is_err());

        // Has required arg
        let result = validate_args(&def, &serde_json::json!({"cmd": "ls"}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_args_no_required() {
        let def = ToolDefinition {
            name: "test".into(),
            description: "test tool".into(),
            parameters: serde_json::json!({}),
        };
        assert!(validate_args(&def, &serde_json::json!({})).is_ok());
    }
}
