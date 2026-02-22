//! Multi-Agent Orchestrator — manages multiple agents and their interactions.
//!
//! Supports:
//! - Named agents with independent configs, tools, memory
//! - Message routing to specific agents
//! - Agent-to-agent delegation (Agent A asks Agent B for help)
//! - Broadcast messages to all agents
//! - Agent roles and specializations

use bizclaw_core::error::Result;
use std::collections::HashMap;

use crate::Agent;

/// A named agent instance with metadata.
pub struct NamedAgent {
    pub agent: Agent,
    pub name: String,
    pub role: String,
    pub description: String,
    pub active: bool,
    pub message_count: u64,
}

/// Multi-Agent Orchestrator — manages a pool of agents.
pub struct Orchestrator {
    agents: HashMap<String, NamedAgent>,
    default_agent: Option<String>,
    /// Inter-agent message log.
    pub message_log: Vec<AgentMessage>,
}

/// A message between agents or from user.
#[derive(Clone)]
pub struct AgentMessage {
    pub from: String,
    pub to: String,
    pub content: String,
    pub response: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Orchestrator {
    /// Create a new empty orchestrator.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            default_agent: None,
            message_log: Vec::new(),
        }
    }

    /// Add an agent to the orchestrator.
    pub fn add_agent(&mut self, name: &str, role: &str, description: &str, agent: Agent) {
        let is_first = self.agents.is_empty();
        self.agents.insert(
            name.to_string(),
            NamedAgent {
                agent,
                name: name.to_string(),
                role: role.to_string(),
                description: description.to_string(),
                active: true,
                message_count: 0,
            },
        );
        if is_first {
            self.default_agent = Some(name.to_string());
        }
    }

    /// Remove an agent.
    pub fn remove_agent(&mut self, name: &str) -> bool {
        let removed = self.agents.remove(name).is_some();
        if self.default_agent.as_deref() == Some(name) {
            self.default_agent = self.agents.keys().next().cloned();
        }
        removed
    }

    /// Set the default agent.
    pub fn set_default(&mut self, name: &str) {
        if self.agents.contains_key(name) {
            self.default_agent = Some(name.to_string());
        }
    }

    /// Send a message to a specific agent.
    pub async fn send_to(&mut self, agent_name: &str, message: &str) -> Result<String> {
        let named = self.agents.get_mut(agent_name).ok_or_else(|| {
            bizclaw_core::error::BizClawError::Config(format!("Agent '{}' not found", agent_name))
        })?;

        named.message_count += 1;
        let response = named.agent.process(message).await?;

        self.message_log.push(AgentMessage {
            from: "user".to_string(),
            to: agent_name.to_string(),
            content: message.to_string(),
            response: Some(response.clone()),
            timestamp: chrono::Utc::now(),
        });

        Ok(response)
    }

    /// Send to the default agent.
    pub async fn send(&mut self, message: &str) -> Result<String> {
        let default = self.default_agent.clone().ok_or_else(|| {
            bizclaw_core::error::BizClawError::Config("No default agent configured".to_string())
        })?;
        self.send_to(&default, message).await
    }

    /// Agent-to-agent delegation — one agent asks another for help.
    pub async fn delegate(
        &mut self,
        from_agent: &str,
        to_agent: &str,
        task: &str,
    ) -> Result<String> {
        // Process the task with the target agent
        let to = self.agents.get_mut(to_agent).ok_or_else(|| {
            bizclaw_core::error::BizClawError::Config(format!(
                "Target agent '{}' not found",
                to_agent
            ))
        })?;

        to.message_count += 1;
        let delegate_prompt = format!(
            "[Delegation from agent '{from_agent}']\n\
             Task: {task}\n\
             Please process this task and return a clear result."
        );
        let response = to.agent.process(&delegate_prompt).await?;

        self.message_log.push(AgentMessage {
            from: from_agent.to_string(),
            to: to_agent.to_string(),
            content: task.to_string(),
            response: Some(response.clone()),
            timestamp: chrono::Utc::now(),
        });

        Ok(response)
    }

    /// Broadcast a message to all active agents and collect responses.
    pub async fn broadcast(&mut self, message: &str) -> Vec<(String, Result<String>)> {
        let agent_names: Vec<String> = self.agents.keys().cloned().collect();
        let mut results = Vec::new();

        for name in agent_names {
            let result = self.send_to(&name, message).await;
            results.push((name, result));
        }

        results
    }

    /// List all agents with their status.
    pub fn list_agents(&self) -> Vec<serde_json::Value> {
        self.agents
            .values()
            .map(|a| {
                serde_json::json!({
                    "name": a.name,
                    "role": a.role,
                    "description": a.description,
                    "active": a.active,
                    "provider": a.agent.provider_name(),
                    "tools": a.agent.tool_count(),
                    "messages_processed": a.message_count,
                    "conversation_length": a.agent.conversation().len(),
                    "is_default": self.default_agent.as_deref() == Some(&a.name),
                })
            })
            .collect()
    }

    /// Get total agent count.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Get the default agent name.
    pub fn default_agent_name(&self) -> Option<&str> {
        self.default_agent.as_deref()
    }

    /// Get recent message log (last N entries).
    pub fn recent_messages(&self, limit: usize) -> Vec<serde_json::Value> {
        self.message_log
            .iter()
            .rev()
            .take(limit)
            .map(|m| {
                serde_json::json!({
                    "from": m.from,
                    "to": m.to,
                    "content": &m.content[..m.content.len().min(200)],
                    "response": m.response.as_ref().map(|r| &r[..r.len().min(200)]),
                    "timestamp": m.timestamp.to_rfc3339(),
                })
            })
            .collect()
    }

    /// Get a mutable reference to an agent.
    pub fn get_agent_mut(&mut self, name: &str) -> Option<&mut Agent> {
        self.agents.get_mut(name).map(|a| &mut a.agent)
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}
