//! MCP Screen Widget Implementation

pub mod design;

use makepad_widgets::*;
use moly_data::{McpServersConfig, Store};

/// Types of toggle switches in the MCP settings
enum ToggleType {
    ServersEnabled,
    DangerousMode,
}

#[derive(Live, LiveHook, Widget)]
pub struct McpApp {
    #[deref]
    pub view: View,

    /// Local copy of the MCP servers configuration
    #[rust]
    mcp_servers_config: McpServersConfig,

    /// Whether the widget has been initialized with data from Store
    #[rust]
    initialized: bool,
}

impl Widget for McpApp {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        let editor = self.widget(ids!(mcp_code_view));

        // Initialize on first load or if editor is empty
        if !self.initialized || editor.text().is_empty() {
            if let Some(store) = scope.data.get::<Store>() {
                self.initialized = true;
                let mut config = store.get_mcp_servers_config().clone();

                // Ensure local config matches Store state
                config.enabled = store.preferences.get_mcp_servers_enabled();
                config.dangerous_mode_enabled =
                    store.preferences.get_mcp_servers_dangerous_mode_enabled();

                self.set_mcp_servers_config(cx, config);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Apply dark mode to all widgets that support it
        if let Some(store) = scope.data.get::<Store>() {
            let dark_mode_value = if store.is_dark_mode() { 1.0 } else { 0.0 };

            // Main container background
            self.view.apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode_value) }
            });

            // Header labels
            self.view.label(ids!(title_label)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode_value) }
            });
            self.view.label(ids!(subtitle_label)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode_value) }
            });

            // Settings panel labels
            self.view.label(ids!(enable_label)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode_value) }
            });
            self.view
                .label(ids!(instructions_label))
                .apply_over(cx, live! {
                    draw_text: { dark_mode: (dark_mode_value) }
                });

            // Danger mode labels
            self.view.label(ids!(danger_label)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode_value) }
            });
            self.view.label(ids!(danger_warning)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode_value) }
            });

            // Status message
            self.view.label(ids!(save_status)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode_value) }
            });

            // Save button
            self.view.view(ids!(save_button)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode_value) }
            });
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl McpApp {
    /// Update the MCP servers configuration and sync UI elements
    fn set_mcp_servers_config(&mut self, cx: &mut Cx, config: McpServersConfig) {
        self.mcp_servers_config = config;

        self.sync_json_display(cx);

        // Sync toggle switch states using the 'active' animator
        self.check_box(ids!(servers_enabled_switch))
            .set_active(cx, self.mcp_servers_config.enabled);
        self.check_box(ids!(dangerous_mode_switch))
            .set_active(cx, self.mcp_servers_config.dangerous_mode_enabled);
    }

    /// Sync the JSON code editor display with the current config
    fn sync_json_display(&mut self, cx: &mut Cx) {
        let display_json = self
            .mcp_servers_config
            .to_json()
            .unwrap_or_else(|_| "{}".to_string());
        self.widget(ids!(mcp_code_view)).set_text(cx, &display_json);
    }

    /// Show a status message (success or error)
    fn show_status(&mut self, cx: &mut Cx, message: &str, is_error: bool) {
        self.label(ids!(save_status)).set_text(cx, message);
        let is_error_value = if is_error { 1.0 } else { 0.0 };
        self.label(ids!(save_status)).apply_over(
            cx,
            live! {
                draw_text: { is_error: (is_error_value) }
            },
        );
    }

    /// Handle toggle switch changes (common logic for servers_enabled and dangerous_mode)
    fn handle_toggle_change(
        &mut self,
        cx: &mut Cx,
        scope: &mut Scope,
        toggle_type: ToggleType,
        enabled: bool,
    ) {
        // Update local config
        match toggle_type {
            ToggleType::ServersEnabled => self.mcp_servers_config.enabled = enabled,
            ToggleType::DangerousMode => self.mcp_servers_config.dangerous_mode_enabled = enabled,
        }

        // Update JSON display to reflect the change
        self.sync_json_display(cx);

        // Sync to Store
        if let Some(store) = scope.data.get_mut::<Store>() {
            match toggle_type {
                ToggleType::ServersEnabled => store.set_mcp_servers_enabled(enabled),
                ToggleType::DangerousMode => store.set_mcp_servers_dangerous_mode_enabled(enabled),
            }
        }
        self.redraw(cx);
    }
}

impl WidgetMatchEvent for McpApp {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // Handle save button click
        if self.view(ids!(save_button)).finger_up(actions).is_some() {
            let json_text = self.widget(ids!(mcp_code_view)).text();

            match McpServersConfig::from_json(&json_text) {
                Ok(config) => {
                    if let Some(store) = scope.data.get_mut::<Store>() {
                        match store.update_mcp_servers_from_json(&json_text) {
                            Ok(()) => {
                                // Also sync the enabled/dangerous mode flags
                                store.set_mcp_servers_enabled(config.enabled);
                                store.set_mcp_servers_dangerous_mode_enabled(
                                    config.dangerous_mode_enabled,
                                );

                                // Update local config
                                self.set_mcp_servers_config(cx, config);

                                // Show success message
                                self.show_status(cx, "Configuration saved!", false);
                                self.redraw(cx);
                            }
                            Err(e) => {
                                // Show error message
                                self.show_status(cx, &format!("Error: {}", e), true);
                                self.redraw(cx);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Show JSON parse error
                    self.show_status(cx, &format!("Invalid JSON: {}", e), true);
                    self.redraw(cx);
                }
            }
        }

        // Handle servers enabled switch toggle
        if let Some(enabled) = self.check_box(ids!(servers_enabled_switch)).changed(actions) {
            self.handle_toggle_change(cx, scope, ToggleType::ServersEnabled, enabled);
        }

        // Handle dangerous mode switch toggle
        if let Some(enabled) = self.check_box(ids!(dangerous_mode_switch)).changed(actions) {
            self.handle_toggle_change(cx, scope, ToggleType::DangerousMode, enabled);
        }
    }
}
