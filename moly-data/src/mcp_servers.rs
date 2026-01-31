use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Represents an input configuration for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub description: String,
    #[serde(default)]
    pub password: bool,
}

/// Represents an MCP server configuration following the standard format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    // Stdio transport fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, String>,

    // HTTP/SSE transport fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transport_type: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub headers: IndexMap<String, String>,

    // Optional extras
    #[serde(default = "default_enabled", skip_serializing_if = "is_default_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

fn default_enabled() -> bool {
    true
}

fn is_default_enabled(enabled: &bool) -> bool {
    *enabled
}

impl McpServer {
    /// Create a new stdio-based MCP server
    pub fn stdio(command: String, args: Vec<String>) -> Self {
        Self {
            command: Some(command),
            args,
            env: IndexMap::new(),
            url: None,
            transport_type: None,
            headers: IndexMap::new(),
            enabled: true,
            working_directory: None,
        }
    }

    /// Create a new HTTP-based MCP server
    pub fn http(url: String) -> Self {
        Self {
            command: None,
            args: Vec::new(),
            env: IndexMap::new(),
            url: Some(url),
            transport_type: Some("http".to_string()),
            headers: IndexMap::new(),
            enabled: true,
            working_directory: None,
        }
    }

    /// Create a new SSE-based MCP server
    pub fn sse(url: String) -> Self {
        Self {
            command: None,
            args: Vec::new(),
            env: IndexMap::new(),
            url: Some(url),
            transport_type: Some("sse".to_string()),
            headers: IndexMap::new(),
            enabled: true,
            working_directory: None,
        }
    }

    /// Check if this is a stdio transport
    pub fn is_stdio(&self) -> bool {
        self.command.is_some()
    }

    /// Check if this is an HTTP/SSE transport
    pub fn is_network(&self) -> bool {
        self.url.is_some()
    }

    /// Set environment variables for stdio transport
    pub fn with_env(mut self, env: IndexMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Set working directory for stdio transport
    pub fn with_working_directory(mut self, working_directory: String) -> Self {
        self.working_directory = Some(working_directory);
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Convert this server configuration to a transport for the MCP manager
    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_transport(&self) -> Option<moly_kit::prelude::McpTransport> {
        use moly_kit::prelude::McpTransport;

        if let Some(command_str) = &self.command {
            let mut command = tokio::process::Command::new(command_str);
            command.args(&self.args);

            for (key, value) in &self.env {
                command.env(key, value);
            }

            if let Some(working_dir) = &self.working_directory {
                command.current_dir(working_dir);
            }

            Some(McpTransport::Stdio(command))
        } else if let Some(url) = &self.url {
            match self.transport_type.as_deref() {
                Some("sse") => Some(McpTransport::Sse(url.clone())),
                _ => Some(McpTransport::Http(url.clone())),
            }
        } else {
            None
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn to_transport(&self) -> Option<()> {
        None
    }
}

fn default_mcp_servers_enabled() -> bool {
    true
}

fn default_dangerous_mode_enabled() -> bool {
    false
}

/// Represents the complete MCP servers configuration (follows MCP standard format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersConfig {
    pub servers: IndexMap<String, McpServer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputConfig>,
    #[serde(default = "default_mcp_servers_enabled")]
    pub enabled: bool,
    #[serde(default = "default_dangerous_mode_enabled")]
    pub dangerous_mode_enabled: bool,
}

impl Default for McpServersConfig {
    fn default() -> Self {
        Self {
            servers: IndexMap::new(),
            inputs: Vec::new(),
            enabled: true,
            dangerous_mode_enabled: false,
        }
    }
}

impl McpServersConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_server(&mut self, id: String, server: McpServer) {
        self.servers.insert(id, server);
    }

    pub fn remove_server(&mut self, id: &str) {
        self.servers.shift_remove(id);
    }

    pub fn get_server(&self, id: &str) -> Option<&McpServer> {
        self.servers.get(id)
    }

    pub fn list_enabled_servers(&self) -> impl Iterator<Item = (&String, &McpServer)> {
        self.servers.iter().filter(|(_, server)| server.enabled)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Create a sample configuration with example servers
    pub fn create_sample() -> Self {
        let mut config = Self::new();

        // HTTP server example (enabled by default)
        config.add_server(
            "my-mcp-server".to_string(),
            McpServer::http("http://localhost:8931".to_string()),
        );

        // Filesystem server (stdio, disabled)
        config.add_server(
            "filesystem".to_string(),
            McpServer::stdio(
                "npx".to_string(),
                vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    "/Users/username/Desktop".to_string(),
                ],
            )
            .with_enabled(false),
        );

        config
    }
}
