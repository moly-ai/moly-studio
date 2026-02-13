//! MCP Screen UI Design

use makepad_widgets::*;

use super::McpApp;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use moly_widgets::theme::*;
    use crate::code_view::MolyCodeView;

    // Toggle switch - using standard CheckBox with minimal customization
    McpSwitch = <CheckBox> {
        width: 40, height: 20
        label_walk: { width: 0 }
    }

    SaveButton = <View> {
        width: Fit, height: Fit
        cursor: Hand
        padding: {left: 20, right: 20, top: 10, bottom: 10}
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance radius: 4.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius);

                let light_base = vec4(0.231, 0.510, 0.965, 1.0);  // #3b82f6
                let light_hover = vec4(0.145, 0.388, 0.922, 1.0); // #2563eb
                let dark_base = vec4(0.145, 0.388, 0.922, 1.0);   // #2563eb
                let dark_hover = vec4(0.114, 0.306, 0.847, 1.0);  // #1d4ed8

                let base = mix(light_base, dark_base, self.dark_mode);
                let hovered = mix(light_hover, dark_hover, self.dark_mode);
                let color = mix(base, hovered, self.hover);
                sdf.fill(color);
                return sdf.result;
            }
        }
        animator: {
            hover = {
                default: off
                off = { from: {all: Forward {duration: 0.15}} apply: {draw_bg: {hover: 0.0}} }
                on = { from: {all: Forward {duration: 0.15}} apply: {draw_bg: {hover: 1.0}} }
            }
        }
        <Label> {
            text: "Save and restart servers"
            draw_text: {
                text_style: <THEME_FONT_BOLD>{ font_size: 11.0 }
                color: #ffffff
            }
        }
    }

    ToggleRow = <View> {
        width: Fill, height: Fit
        flow: Right, spacing: 12
        align: {y: 0.5}
    }

    pub McpApp = {{McpApp}} {
        width: Fill, height: Fill
        flow: Down
        padding: 20
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix(#f5f7fa, #0f172a, self.dark_mode);
            }
        }

        // Header
        <View> {
            width: Fill, height: Fit
            flow: Down, spacing: 8
            padding: {bottom: 20}

            title_label = <Label> {
                text: "MCP Servers"
                draw_text: {
                    instance dark_mode: 0.0
                    fn get_color(self) -> vec4 {
                        return mix(#1f2937, #f1f5f9, self.dark_mode);
                    }
                    text_style: <THEME_FONT_BOLD>{ font_size: 24.0 }
                }
            }
            subtitle_label = <Label> {
                text: "Manage MCP servers and tools"
                draw_text: {
                    instance dark_mode: 0.0
                    fn get_color(self) -> vec4 {
                        return mix(#6b7280, #94a3b8, self.dark_mode);
                    }
                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                }
            }
        }

        // Main content - horizontal split
        <View> {
            width: Fill, height: Fill
            flow: Right, spacing: 20

            // Left: Code editor
            <View> {
                width: 550, height: Fill
                flow: Down, spacing: 10

                // Editor container
                <RoundedView> {
                    width: Fill, height: Fill
                    draw_bg: {
                        color: #1d2330
                        border_radius: 6.0
                    }
                    mcp_code_view = <MolyCodeView> {}
                }

                // Save button row
                <View> {
                    width: Fill, height: Fit
                    align: {x: 1.0}
                    save_button = <SaveButton> {}
                }
            }

            // Right: Settings panel
            <View> {
                width: Fill, height: Fill
                flow: Down, spacing: 20
                padding: {top: 10}

                // Enable MCP Servers toggle
                <ToggleRow> {
                    enable_label = <Label> {
                        text: "Enable MCP Servers"
                        draw_text: {
                            instance dark_mode: 0.0
                            fn get_color(self) -> vec4 {
                                return mix(#1f2937, #f1f5f9, self.dark_mode);
                            }
                            text_style: <THEME_FONT_BOLD>{ font_size: 12.0 }
                        }
                    }
                    servers_enabled_switch = <McpSwitch> {
                        animator: { active = { default: on } }
                    }
                }

                // Instructions
                <View> {
                    width: Fill, height: Fit
                    flow: Down, spacing: 8

                    instructions_label = <Label> {
                        width: Fill
                        text: "Add new servers by editing the JSON. You can copy your configuration from Claude Desktop or VSCode.\n\nAdd \"enabled\": false to disable a specific server."
                        draw_text: {
                            wrap: Word
                            instance dark_mode: 0.0
                            fn get_color(self) -> vec4 {
                                return mix(#4b5563, #9ca3af, self.dark_mode);
                            }
                            text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                        }
                    }
                }

                // Dangerous Mode section
                <View> {
                    width: Fill, height: Fit
                    flow: Down, spacing: 8
                    margin: {top: 10}

                    <ToggleRow> {
                        danger_label = <Label> {
                            text: "Dangerous Mode"
                            draw_text: {
                                instance dark_mode: 0.0
                                fn get_color(self) -> vec4 {
                                    // Light: #ef4444, Dark: #fca5a5 (lighter red for dark mode)
                                    return mix(#ef4444, #fca5a5, self.dark_mode);
                                }
                                text_style: <THEME_FONT_BOLD>{ font_size: 12.0 }
                            }
                        }
                        dangerous_mode_switch = <McpSwitch> {
                            animator: { active = { default: off } }
                        }
                    }

                    danger_warning = <Label> {
                        width: Fill
                        text: "WARNING: This mode automatically approves ALL tool calls without asking for permission. Only enable if you trust all configured MCP servers completely."
                        draw_text: {
                            wrap: Word
                            instance dark_mode: 0.0
                            fn get_color(self) -> vec4 {
                                // Light: #f87171, Dark: #fecaca (lighter red for dark mode)
                                return mix(#f87171, #fecaca, self.dark_mode);
                            }
                            text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                        }
                    }
                }

                // Status message
                <View> {
                    width: Fill, height: Fit
                    margin: {top: 10}

                    save_status = <Label> {
                        text: ""
                        draw_text: {
                            instance dark_mode: 0.0
                            instance is_error: 0.0
                            fn get_color(self) -> vec4 {
                                // Success colors: Light: #059669, Dark: #34d399
                                let success_color = mix(#059669, #34d399, self.dark_mode);
                                // Error colors: Light: #dc2626, Dark: #f87171
                                let error_color = mix(#dc2626, #f87171, self.dark_mode);
                                return mix(success_color, error_color, self.is_error);
                            }
                            text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                        }
                    }
                }
            }
        }
    }
}
