use makepad_widgets::*;
use moly_kit::prelude::*;
use std::sync::{Arc, Mutex};

use crate::chats::Chats;
use crate::mcp_servers::McpServersConfig;
use crate::moly_client::MolyClient;
use crate::preferences::Preferences;
use crate::providers_manager::ProvidersManager;

/// Actions that can be dispatched to modify the Store
#[derive(Clone, Debug, DefaultNone)]
pub enum StoreAction {
    /// Toggle dark mode
    ToggleDarkMode,
    /// Set dark mode explicitly
    SetDarkMode(bool),
    /// Toggle sidebar expanded/collapsed
    ToggleSidebar,
    /// Set sidebar expanded state explicitly
    SetSidebarExpanded(bool),
    /// Navigate to a specific view
    Navigate(String),
    /// No action
    None,
}

/// Central state container for the application
///
/// The Store holds all shared application state and is passed down
/// to widgets via Makepad's Scope mechanism.
///
/// # Usage
///
/// In App's handle_event:
/// ```rust,ignore
/// let scope = &mut Scope::with_data(&mut self.store);
/// self.ui.handle_event(cx, event, scope);
/// ```
///
/// In child widgets:
/// ```rust,ignore
/// let store = scope.data.get::<Store>().unwrap();
/// // Read from store
///
/// let store = scope.data.get_mut::<Store>().unwrap();
/// // Modify store
/// ```
pub struct Store {
    /// User preferences (persisted to disk)
    pub preferences: Preferences,

    /// Chat sessions management
    pub chats: Chats,

    /// The ChatController for the current chat (from aitk)
    pub chat_controller: Option<Arc<Mutex<ChatController>>>,

    /// Multi-provider client manager
    pub providers_manager: ProvidersManager,

    /// Moly Server client for model discovery and downloads
    pub moly_client: MolyClient,

    /// Whether the Store has been fully initialized
    pub initialized: bool,
}

impl Default for Store {
    fn default() -> Self {
        // WARNING: This creates a Store with default preferences (no API keys!)
        // Use Store::load() instead to load from disk.
        Self {
            preferences: Preferences::default(),
            chats: Chats::new(),
            chat_controller: None,
            providers_manager: ProvidersManager::new(),
            moly_client: MolyClient::new(),
            initialized: false,
        }
    }
}

impl Store {
    /// Create a new Store by loading preferences from disk
    pub fn load() -> Self {
        let preferences = Preferences::load();

        // Create a ChatController with basic async spawner
        let chat_controller = ChatController::new_arc();
        {
            let mut controller = chat_controller.lock().unwrap();
            controller.set_basic_spawner();
        }

        // Create ProvidersManager and configure with enabled providers
        let mut providers_manager = ProvidersManager::new();
        let enabled_providers: Vec<_> = preferences.get_enabled_providers();
        providers_manager.configure_providers(&enabled_providers);

        // Load chats from disk
        let chats = Chats::load();

        // Create MolyClient for model discovery
        let moly_client = MolyClient::new();

        Self {
            preferences,
            chats,
            chat_controller: Some(chat_controller),
            providers_manager,
            moly_client,
            initialized: true,
        }
    }

    /// Reconfigure providers manager when provider settings change
    pub fn reconfigure_providers(&mut self) {
        let enabled_providers: Vec<_> = self.preferences.get_enabled_providers();
        self.providers_manager.configure_providers(&enabled_providers);
    }

    /// Get a reference to the ChatController
    pub fn get_chat_controller(&self) -> Option<Arc<Mutex<ChatController>>> {
        self.chat_controller.clone()
    }

    /// Check if dark mode is enabled
    pub fn is_dark_mode(&self) -> bool {
        self.preferences.dark_mode
    }

    /// Set dark mode state
    pub fn set_dark_mode(&mut self, dark_mode: bool) {
        self.preferences.set_dark_mode(dark_mode);
    }

    /// Toggle dark mode
    pub fn toggle_dark_mode(&mut self) {
        self.set_dark_mode(!self.is_dark_mode());
    }

    /// Check if sidebar is expanded
    pub fn is_sidebar_expanded(&self) -> bool {
        self.preferences.sidebar_expanded
    }

    /// Set sidebar expanded state
    pub fn set_sidebar_expanded(&mut self, expanded: bool) {
        self.preferences.set_sidebar_expanded(expanded);
    }

    /// Toggle sidebar expanded/collapsed
    pub fn toggle_sidebar(&mut self) {
        self.set_sidebar_expanded(!self.is_sidebar_expanded());
    }

    /// Get current view name
    pub fn current_view(&self) -> &str {
        &self.preferences.current_view
    }

    /// Set current view
    pub fn set_current_view(&mut self, view: &str) {
        self.preferences.set_current_view(view);
    }

    /// Handle a StoreAction and update state accordingly
    pub fn handle_action(&mut self, action: &StoreAction) {
        match action {
            StoreAction::ToggleDarkMode => {
                self.toggle_dark_mode();
            }
            StoreAction::SetDarkMode(dark_mode) => {
                self.set_dark_mode(*dark_mode);
            }
            StoreAction::ToggleSidebar => {
                self.toggle_sidebar();
            }
            StoreAction::SetSidebarExpanded(expanded) => {
                self.set_sidebar_expanded(*expanded);
            }
            StoreAction::Navigate(view) => {
                self.set_current_view(view);
            }
            StoreAction::None => {}
        }
    }

    // =========================================================================
    // MCP Server Configuration Methods
    // =========================================================================

    /// Get MCP servers config reference
    pub fn get_mcp_servers_config(&self) -> &McpServersConfig {
        &self.preferences.mcp_servers_config
    }

    /// Get MCP servers config as JSON
    pub fn get_mcp_servers_config_json(&self) -> String {
        self.preferences.get_mcp_servers_config_json()
    }

    /// Update MCP servers from JSON
    pub fn update_mcp_servers_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        self.preferences.update_mcp_servers_from_json(json)
    }

    /// Set MCP servers enabled state
    pub fn set_mcp_servers_enabled(&mut self, enabled: bool) {
        self.preferences.set_mcp_servers_enabled(enabled);
    }

    /// Set dangerous mode enabled
    pub fn set_mcp_servers_dangerous_mode_enabled(&mut self, enabled: bool) {
        self.preferences.set_mcp_servers_dangerous_mode_enabled(enabled);
    }

    /// Creates a new MCP tool manager and loads servers asynchronously
    /// Returns the manager immediately, loading happens in the background
    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_and_load_mcp_tool_manager(&self) -> moly_kit::prelude::McpManagerClient {
        use moly_kit::aitk::utils::asynchronous::spawn;
        use moly_kit::prelude::McpManagerClient;

        let tool_manager = McpManagerClient::new();

        // Check if MCP servers are globally enabled
        if !self.preferences.get_mcp_servers_enabled() {
            return tool_manager;
        }

        let mcp_config = self.get_mcp_servers_config().clone();
        tool_manager.set_dangerous_mode_enabled(mcp_config.dangerous_mode_enabled);
        let tool_manager_clone = tool_manager.clone();

        spawn(async move {
            for (server_id, server_config) in mcp_config.list_enabled_servers() {
                if let Some(transport) = server_config.to_transport() {
                    match tool_manager_clone.add_server(server_id, transport).await {
                        Ok(()) => {
                            ::log::debug!("Successfully added MCP server: {}", server_id);
                        }
                        Err(e) => {
                            ::log::error!("Failed to add MCP server '{}': {}", server_id, e);
                        }
                    }
                }
            }
        });

        tool_manager
    }

    /// Creates a new MCP tool manager (wasm version - no actual server loading)
    #[cfg(target_arch = "wasm32")]
    pub fn create_and_load_mcp_tool_manager(&self) -> moly_kit::prelude::McpManagerClient {
        moly_kit::prelude::McpManagerClient::new()
    }
}
