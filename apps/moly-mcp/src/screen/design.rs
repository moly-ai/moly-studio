//! MCP Screen UI Design

use makepad_widgets::*;

use super::McpApp;
use crate::code_view::MolyCodeView;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use moly_widgets::theme::*;
    use crate::code_view::MolyCodeView;

    // Toggle switch styled like a modern switch
    McpSwitch = <CheckBox> {
        width: 40, height: 20
        label_walk: { width: 0 }
        draw_check: {
            instance radius: 4.0
            instance on_color: #4ade80
            instance off_color: #64748b

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let sz = self.rect_size;

                // Track background
                sdf.box(1.0, 1.0, sz.x - 2.0, sz.y - 2.0, self.radius);
                let bg_color = mix(self.off_color, self.on_color, self.selected);
                sdf.fill(bg_color);

                // Knob
                let knob_size = sz.y - 6.0;
                let knob_x = mix(3.0, sz.x - knob_size - 3.0, self.selected);
                sdf.circle(knob_x + knob_size * 0.5, sz.y * 0.5, knob_size * 0.5);
                sdf.fill(#ffffff);

                return sdf.result;
            }
        }
        animator: {
            selected = {
                default: off
                off = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_check: { selected: 0.0 } }
                }
                on = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_check: { selected: 1.0 } }
                }
            }
        }
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
                text_style: <THEME_FONT_SEMIBOLD>{ font_size: 11.0 }
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
                        radius: 6.0
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
                            text_style: <THEME_FONT_SEMIBOLD>{ font_size: 12.0 }
                        }
                    }
                    servers_enabled_switch = <McpSwitch> {
                        animator: { selected = { default: on } }
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
                                text_style: <THEME_FONT_SEMIBOLD>{ font_size: 12.0 }
                                color: #ef4444
                            }
                        }
                        dangerous_mode_switch = <McpSwitch> {
                            animator: { selected = { default: off } }
                        }
                    }

                    danger_warning = <Label> {
                        width: Fill
                        text: "WARNING: This mode automatically approves ALL tool calls without asking for permission. Only enable if you trust all configured MCP servers completely."
                        draw_text: {
                            wrap: Word
                            text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                            color: #f87171
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
                            fn get_color(self) -> vec4 {
                                return mix(#059669, #34d399, self.dark_mode);
                            }
                            text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                        }
                    }
                }
            }
        }
    }
}
