use egui::{Color32, CornerRadius, Stroke, Style, Vec2, Visuals};
use ron;
use serde::{Deserialize, Serialize};

#[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
use std::fs;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaseThemeChoice {
    Light,
    Dark,
    // Custom, // We can infer 'Custom' if the settings deviate significantly from Light/Dark defaults
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeSettings {
    pub base_theme: BaseThemeChoice,

    // --- General UI Colors ---
    pub primary_text_color: Color32,
    pub secondary_text_color: Color32, // For weaker/less important text, placeholders
    pub strong_text_color: Color32,    // For emphasized text, headings
    pub hyperlink_color: Color32,
    pub window_background: Color32,
    pub panel_background: Color32, // For ui.group, side panels, frames
    pub code_bg_color: Color32,    // Background for code blocks/editor view

    // --- General Accent Colors ---
    pub accent_color_primary: Color32, // Main brand/highlight color
    pub accent_color_secondary: Color32, // Secondary highlight or related action
    pub accent_color_tertiary: Color32, // Less prominent accent
    pub accent_color_positive: Color32, // For positive feedback, not just success messages
    pub accent_color_negative: Color32, // For negative feedback, not just error messages

    // Widget States (buttons, sliders, checkboxes, etc.)
    pub widget_text_color: Color32, // Text color on interactive widgets
    pub widget_fill_interactive: Color32, // Normal state
    pub widget_fill_hovered: Color32,
    pub widget_fill_active: Color32, // Clicked/dragged state
    pub widget_fill_disabled: Color32,

    // -- Widget Stroke Styles ---
    pub bg_widget_stroke_interactive: Stroke,
    pub bg_widget_stroke_hovered: Stroke,
    pub bg_widget_stroke_active: Stroke,
    pub bg_widget_stroke_disabled: Stroke,

    pub fg_widget_stroke_interactive: Stroke, // Foreground stroke for normal state
    pub fg_widget_stroke_hovered: Stroke,
    pub fg_widget_stroke_active: Stroke,
    pub fg_widget_stroke_disabled: Stroke,

    pub selection_stroke: Stroke,

    // Semantic Colors (for messages, highlights)
    pub error_fg_color: Color32,
    pub error_bg_color: Color32, // Optional background for error sections
    pub warn_fg_color: Color32,
    pub warn_bg_color: Color32,
    pub success_fg_color: Color32,
    pub success_bg_color: Color32,
    pub info_fg_color: Color32, // For general informational messages
    pub info_bg_color: Color32,

    // --- Emulator Specific Colors ---
    // General Emulator Highlights
    pub pc_line_bg: Color32, // Background for the line where PC is in memory/editor
    pub breakpoint_bg: Color32, // Background for breakpoint lines
    pub highlight_bg: Color32, // Background for temporarily highlighted items (e.g., jump target)

    // Memory Pane
    pub memory_address_color: Color32,
    pub memory_label_color: Color32,
    pub memory_value_color: Color32,
    pub memory_ascii_color: Color32,
    pub memory_instruction_color: Color32,
    pub memory_os_code_bg: Color32,
    pub memory_os_data_bg: Color32,
    pub memory_user_code_bg: Color32,
    pub memory_user_data_bg: Color32,
    pub memory_stack_bg: Color32,
    pub memory_heap_bg: Color32, // For dynamically allocated memory if supported/visualized
    pub memory_device_registers_bg: Color32,
    pub memory_unused_bg: Color32, // For memory regions not otherwise categorized
    pub memory_zone_label_color: Color32, // For legend or overlay text on zones

    // Registers Pane
    pub register_name_color: Color32, // General purpose registers (R0-R7)
    pub register_value_color: Color32,
    pub register_decoded_name_color: Color32, // For fields within a decoded register value
    pub register_decoded_value_color: Color32,
    pub register_special_purpose_name_color: Color32, // PC, IR, PSR etc.
    pub register_special_purpose_value_color: Color32,
    pub register_device_name_color: Color32, // KBSR, KBDR, DSR, DDR etc.
    pub register_device_value_color: Color32,

    // CPU State Pane
    pub cpu_state_active_color: Color32, // For the current CPU cycle step text/icon
    pub cpu_state_inactive_color: Color32, // For other CPU cycle steps text/icon
    pub cpu_state_description_color: Color32, // For the detailed text in CPU state
    pub cpu_state_data_flow_color: Color32, // For arrows or highlights showing data movement
    pub cpu_state_active_register_highlight: Color32, // Background highlight for active regs in detailed view
    pub cpu_state_active_memory_highlight: Color32, // Background highlight for active mem access in detailed view

    // IO Pane (Terminal)
    pub terminal_text_color: Color32,
    pub terminal_bg_color: Color32,
    pub terminal_cursor_color: Color32,
    pub terminal_selection_bg_color: Color32,
    pub terminal_link_color: Color32, // If terminal supports hyperlinks

    // Editor Pane (Syntax Highlighting)
    pub editor_label_color: Color32,
    pub editor_register_color: Color32,
    pub editor_directive_color: Color32,
    pub editor_opcode_color: Color32,
    pub editor_literal_color: Color32, // General for numbers
    pub editor_string_color: Color32,
    pub editor_char_color: Color32,
    pub editor_comment_color: Color32,
    pub editor_error_underline_color: Color32,
    pub editor_warning_underline_color: Color32,
    pub editor_matching_bracket_bg_color: Color32,
    pub editor_current_line_number_color: Color32,
    pub editor_line_number_color: Color32, // Default line number color

    // Help/Reference Pane Specific
    pub opcode_color: Color32,
    pub help_operand_color: Color32, // For DR, SR, BaseR etc.
    pub help_immediate_color: Color32,
    pub help_offset_color: Color32,
    pub help_binary_layout_fixed_bits_color: Color32, // For '0000' or '1' in binary formats
    pub help_pseudo_code_color: Color32,
    pub help_title_color: Color32,
    pub help_heading_color: Color32,
    pub help_sub_heading_color: Color32,
    pub help_strong_label_color: Color32,
    pub help_italic_label_color: Color32,
    pub help_monospace_color: Color32, // Default for non-semantic monospace
    pub help_code_block_text_color: Color32,
    pub help_collapsible_header_bg_color: Color32,
    pub help_collapsible_header_text_color: Color32,
    pub help_info_list_icon_color: Color32,
    pub help_link_color: Color32,

    // --- Look & Feel ---
    pub item_spacing: Vec2,
    pub widget_rounding: CornerRadius,
    pub window_rounding: CornerRadius,
    pub scroll_bar_width: f32,
    pub window_stroke: Stroke,
    pub panel_stroke: Stroke,
    pub separator_color: Color32,
    pub tooltip_bg_color: Color32,
    pub tooltip_text_color: Color32,
    pub scrollbar_bg_color: Color32,
    pub scrollbar_handle_color: Color32,
    pub scrollbar_handle_hovered_color: Color32,
    pub scrollbar_handle_active_color: Color32,
    pub scrollbar_rounding: CornerRadius,
    pub button_padding: Vec2,
    pub indent_width: f32,
}

impl ThemeSettings {
    pub fn dark_default() -> Self {
        // Try to load from embedded RON file, fallback to hardcoded default if parsing fails
        let ron_bytes = include_bytes!("../assets/themes/dark.ron");
        if let Ok(ron_str) = std::str::from_utf8(ron_bytes) {
            if let Ok(settings) = ron::de::from_str::<ThemeSettings>(ron_str) {
                return settings;
            }
        }

        warn!("Failed to load dark theme from RON file: falling back");

        // Fallback: hardcoded dark theme
        let _vis = Visuals::dark(); // Base egui dark visuals
        Self {
            base_theme: BaseThemeChoice::Dark,

            primary_text_color: Color32::from_gray(220),
            secondary_text_color: Color32::from_gray(160),
            strong_text_color: Color32::WHITE,
            hyperlink_color: Color32::from_rgb(100, 170, 255),
            window_background: Color32::from_gray(20),
            panel_background: Color32::from_gray(30),
            code_bg_color: Color32::from_gray(25),

            accent_color_primary: Color32::from_rgb(0, 150, 255),
            accent_color_secondary: Color32::from_rgb(0, 200, 200),
            accent_color_tertiary: Color32::from_rgb(150, 150, 150),
            accent_color_positive: Color32::from_rgb(50, 200, 50),
            accent_color_negative: Color32::from_rgb(255, 80, 80),

            widget_text_color: Color32::from_gray(230),
            widget_fill_interactive: Color32::from_gray(55),
            widget_fill_hovered: Color32::from_gray(70),
            widget_fill_active: Color32::from_rgb(0, 110, 190),
            widget_fill_disabled: Color32::from_gray(45),
            bg_widget_stroke_interactive: Stroke::new(1.0, Color32::from_gray(80)),
            bg_widget_stroke_hovered: Stroke::new(1.5, Color32::from_gray(100)),
            bg_widget_stroke_active: Stroke::new(1.5, Color32::from_rgb(0, 150, 255)),
            bg_widget_stroke_disabled: Stroke::new(1.0, Color32::from_gray(60)),
            fg_widget_stroke_interactive: Stroke::new(1.0, Color32::from_gray(180)),
            fg_widget_stroke_hovered: Stroke::new(1.5, Color32::from_gray(220)),
            fg_widget_stroke_active: Stroke::new(2.0, Color32::from_rgb(0, 150, 255)),
            fg_widget_stroke_disabled: Stroke::new(1.0, Color32::from_gray(80)),

            selection_stroke: Stroke::new(1.0, Color32::from_gray(100)),

            error_fg_color: Color32::from_rgb(255, 100, 100),
            error_bg_color: Color32::from_rgb(60, 30, 30),
            warn_fg_color: Color32::from_rgb(255, 200, 80),
            warn_bg_color: Color32::from_rgb(70, 55, 30),
            success_fg_color: Color32::from_rgb(100, 220, 100),
            success_bg_color: Color32::from_rgb(30, 60, 30),
            info_fg_color: Color32::from_rgb(120, 180, 255),
            info_bg_color: Color32::from_rgb(30, 45, 70),

            pc_line_bg: Color32::from_rgba_premultiplied(30, 90, 30, 200),
            breakpoint_bg: Color32::from_rgba_premultiplied(100, 30, 30, 200),
            highlight_bg: Color32::from_rgba_premultiplied(90, 70, 160, 150),

            memory_address_color: Color32::from_gray(140),
            memory_label_color: Color32::from_rgb(210, 190, 110),
            memory_value_color: Color32::from_gray(215),
            memory_ascii_color: Color32::from_gray(150),
            memory_instruction_color: Color32::from_rgb(150, 190, 220),
            memory_os_code_bg: Color32::from_rgba_premultiplied(30, 35, 50, 200),
            memory_os_data_bg: Color32::from_rgba_premultiplied(35, 40, 55, 200),
            memory_user_code_bg: Color32::from_rgba_premultiplied(40, 30, 45, 200),
            memory_user_data_bg: Color32::from_rgba_premultiplied(45, 35, 50, 200),
            memory_stack_bg: Color32::from_rgba_premultiplied(50, 45, 30, 200),
            memory_heap_bg: Color32::from_rgba_premultiplied(30, 50, 45, 200),
            memory_device_registers_bg: Color32::from_rgba_premultiplied(55, 30, 30, 200),
            memory_unused_bg: Color32::from_rgba_premultiplied(22, 22, 22, 200),
            memory_zone_label_color: Color32::from_gray(160), // secondary_text_color

            register_name_color: Color32::from_rgb(140, 170, 210),
            register_value_color: Color32::from_gray(215),
            register_decoded_name_color: Color32::from_gray(160), // secondary_text_color
            register_decoded_value_color: Color32::from_gray(220), // primary_text_color
            register_special_purpose_name_color: Color32::from_rgb(160, 190, 230),
            register_special_purpose_value_color: Color32::WHITE, // strong_text_color
            register_device_name_color: Color32::from_rgb(230, 160, 160),
            register_device_value_color: Color32::WHITE, // strong_text_color

            cpu_state_active_color: Color32::from_rgb(80, 200, 80),
            cpu_state_inactive_color: Color32::from_gray(110),
            cpu_state_description_color: Color32::from_gray(195),
            cpu_state_data_flow_color: Color32::from_rgb(0, 200, 200), // accent_color_secondary
            cpu_state_active_register_highlight: Color32::from_rgba_premultiplied(70, 90, 120, 100),
            cpu_state_active_memory_highlight: Color32::from_rgba_premultiplied(90, 120, 70, 100),

            terminal_text_color: Color32::from_gray(220), // primary_text_color
            terminal_bg_color: Color32::from_gray(18),
            terminal_cursor_color: Color32::from_rgb(0, 150, 255), // accent_color_primary
            terminal_selection_bg_color: Color32::from_rgb(0, 150, 255).linear_multiply(0.4),
            terminal_link_color: Color32::from_rgb(100, 170, 255), // hyperlink_color

            editor_label_color: Color32::from_rgb(220, 220, 150),
            editor_register_color: Color32::from_rgb(150, 190, 220),
            editor_directive_color: Color32::from_rgb(190, 150, 220),
            editor_opcode_color: Color32::from_rgb(220, 150, 150),
            editor_literal_color: Color32::from_rgb(150, 220, 150),
            editor_string_color: Color32::from_rgb(200, 180, 130),
            editor_char_color: Color32::from_rgb(200, 180, 130),
            editor_comment_color: Color32::from_gray(120),
            editor_error_underline_color: Color32::from_rgb(255, 100, 100), // error_fg_color
            editor_warning_underline_color: Color32::from_rgb(255, 200, 80), // warn_fg_color
            editor_matching_bracket_bg_color: Color32::from_gray(60),
            editor_current_line_number_color: Color32::WHITE, // strong_text_color
            editor_line_number_color: Color32::from_gray(100),

            opcode_color: Color32::from_rgb(255, 100, 100),
            help_operand_color: Color32::from_rgb(100, 180, 255),
            help_immediate_color: Color32::from_rgb(100, 220, 100),
            help_offset_color: Color32::from_rgb(220, 160, 255),
            help_binary_layout_fixed_bits_color: Color32::from_gray(120),
            help_pseudo_code_color: Color32::from_gray(180),
            help_title_color: Color32::WHITE,
            help_heading_color: Color32::from_gray(230),
            help_sub_heading_color: Color32::from_gray(200),
            help_strong_label_color: Color32::from_gray(225),
            help_italic_label_color: Color32::from_gray(170),
            help_monospace_color: Color32::from_gray(200),
            help_code_block_text_color: Color32::from_gray(210),
            help_collapsible_header_bg_color: Color32::from_gray(40),
            help_collapsible_header_text_color: Color32::from_gray(220),
            help_info_list_icon_color: Color32::from_rgb(0, 150, 255),
            help_link_color: Color32::from_rgb(120, 190, 255),

            item_spacing: Vec2::new(8.0, 7.0),
            widget_rounding: CornerRadius::same(2),
            window_rounding: CornerRadius::same(6),
            scroll_bar_width: 12.0,
            window_stroke: Stroke::new(1.0, Color32::from_gray(50)),
            panel_stroke: Stroke::new(1.0, Color32::from_gray(45)),
            separator_color: Color32::from_gray(50),
            tooltip_bg_color: Color32::from_gray(40),
            tooltip_text_color: Color32::from_gray(220),
            scrollbar_bg_color: Color32::from_gray(30),
            scrollbar_handle_color: Color32::from_gray(70),
            scrollbar_handle_hovered_color: Color32::from_gray(85),
            scrollbar_handle_active_color: Color32::from_gray(100),
            scrollbar_rounding: CornerRadius::same(2),
            button_padding: Vec2::new(10.0, 6.0),
            indent_width: 20.0,
        }
    }
    pub fn light_default() -> Self {
        // Try to load from embedded RON file, fallback to hardcoded default if parsing fails
        let ron_bytes = include_bytes!("../assets/themes/light.ron");
        if let Ok(ron_str) = std::str::from_utf8(ron_bytes) {
            if let Ok(settings) = ron::de::from_str::<ThemeSettings>(ron_str) {
                return settings;
            }
        }

        warn!("Failed to load light theme from RON file: falling back");

        // Fallback: hardcoded light theme
        let _vis = Visuals::light(); // Base egui light visuals
        Self {
            base_theme: BaseThemeChoice::Light,

            primary_text_color: Color32::from_gray(10),
            secondary_text_color: Color32::from_gray(80),
            strong_text_color: Color32::BLACK,
            hyperlink_color: Color32::from_rgb(0, 100, 220),
            window_background: Color32::WHITE,
            panel_background: Color32::from_gray(245),
            code_bg_color: Color32::from_gray(240),

            accent_color_primary: Color32::from_rgb(0, 120, 220),
            accent_color_secondary: Color32::from_rgb(0, 170, 170),
            accent_color_tertiary: Color32::from_gray(100),
            accent_color_positive: Color32::from_rgb(0, 150, 0),
            accent_color_negative: Color32::from_rgb(200, 0, 0),

            widget_text_color: Color32::from_gray(10),
            widget_fill_interactive: Color32::from_gray(230),
            widget_fill_hovered: Color32::from_gray(215),
            widget_fill_active: Color32::from_rgb(0, 90, 180),
            widget_fill_disabled: Color32::from_gray(235),
            bg_widget_stroke_interactive: Stroke::new(1.0, Color32::from_gray(190)),
            bg_widget_stroke_hovered: Stroke::new(1.5, Color32::from_gray(160)),
            bg_widget_stroke_active: Stroke::new(1.5, Color32::from_rgb(0, 120, 220)),
            bg_widget_stroke_disabled: Stroke::new(1.0, Color32::from_gray(210)),
            fg_widget_stroke_interactive: Stroke::new(1.0, Color32::from_gray(80)),
            fg_widget_stroke_hovered: Stroke::new(1.5, Color32::from_gray(40)),
            fg_widget_stroke_active: Stroke::new(2.0, Color32::from_rgb(0, 120, 220)),
            fg_widget_stroke_disabled: Stroke::new(1.0, Color32::from_gray(210)),

            selection_stroke: Stroke::new(1.0, Color32::from_gray(100)),

            error_fg_color: Color32::from_rgb(190, 0, 0),
            error_bg_color: Color32::from_rgb(255, 225, 225),
            warn_fg_color: Color32::from_rgb(170, 110, 0),
            warn_bg_color: Color32::from_rgb(255, 240, 210),
            success_fg_color: Color32::from_rgb(0, 130, 0),
            success_bg_color: Color32::from_rgb(215, 255, 215),
            info_fg_color: Color32::from_rgb(0, 110, 190),
            info_bg_color: Color32::from_rgb(215, 235, 255),

            pc_line_bg: Color32::from_rgba_premultiplied(170, 255, 170, 200),
            breakpoint_bg: Color32::from_rgba_premultiplied(255, 170, 170, 200),
            highlight_bg: Color32::from_rgba_premultiplied(210, 190, 255, 150),

            memory_address_color: Color32::from_gray(60),
            memory_label_color: Color32::from_rgb(140, 110, 0),
            memory_value_color: Color32::from_gray(30),
            memory_ascii_color: Color32::from_gray(90),
            memory_instruction_color: Color32::from_rgb(20, 70, 110),
            memory_os_code_bg: Color32::from_rgba_premultiplied(220, 225, 240, 200),
            memory_os_data_bg: Color32::from_rgba_premultiplied(225, 230, 245, 200),
            memory_user_code_bg: Color32::from_rgba_premultiplied(230, 220, 235, 200),
            memory_user_data_bg: Color32::from_rgba_premultiplied(235, 225, 240, 200),
            memory_stack_bg: Color32::from_rgba_premultiplied(240, 235, 220, 200),
            memory_heap_bg: Color32::from_rgba_premultiplied(220, 240, 235, 200),
            memory_device_registers_bg: Color32::from_rgba_premultiplied(245, 220, 220, 200),
            memory_unused_bg: Color32::from_rgba_premultiplied(242, 242, 242, 200),
            memory_zone_label_color: Color32::from_gray(80), // secondary_text_color

            register_name_color: Color32::from_rgb(30, 80, 150),
            register_value_color: Color32::from_gray(30),
            register_decoded_name_color: Color32::from_gray(80), // secondary_text_color
            register_decoded_value_color: Color32::from_gray(10), // primary_text_color
            register_special_purpose_name_color: Color32::from_rgb(50, 80, 140),
            register_special_purpose_value_color: Color32::BLACK, // strong_text_color
            register_device_name_color: Color32::from_rgb(140, 50, 50),
            register_device_value_color: Color32::BLACK, // strong_text_color

            cpu_state_active_color: Color32::from_rgb(0, 150, 0),
            cpu_state_inactive_color: Color32::from_gray(120),
            cpu_state_description_color: Color32::from_gray(50),
            cpu_state_data_flow_color: Color32::from_rgb(0, 170, 170), // accent_color_secondary
            cpu_state_active_register_highlight: Color32::from_rgba_premultiplied(
                180, 200, 230, 100,
            ),
            cpu_state_active_memory_highlight: Color32::from_rgba_premultiplied(200, 230, 180, 100),

            terminal_text_color: Color32::from_gray(10), // primary_text_color
            terminal_bg_color: Color32::from_gray(252),
            terminal_cursor_color: Color32::from_rgb(0, 120, 220), // accent_color_primary
            terminal_selection_bg_color: Color32::from_rgb(0, 120, 220).linear_multiply(0.4),
            terminal_link_color: Color32::from_rgb(0, 100, 220), // hyperlink_color

            editor_label_color: Color32::from_rgb(150, 130, 0),
            editor_register_color: Color32::from_rgb(20, 70, 110),
            editor_directive_color: Color32::from_rgb(110, 20, 130),
            editor_opcode_color: Color32::from_rgb(130, 20, 20),
            editor_literal_color: Color32::from_rgb(20, 130, 20),
            editor_string_color: Color32::from_rgb(160, 90, 0),
            editor_char_color: Color32::from_rgb(160, 90, 0),
            editor_comment_color: Color32::from_gray(120),
            editor_error_underline_color: Color32::from_rgb(190, 0, 0), // error_fg_color
            editor_warning_underline_color: Color32::from_rgb(170, 110, 0), // warn_fg_color
            editor_matching_bracket_bg_color: Color32::from_gray(200),
            editor_current_line_number_color: Color32::BLACK, // strong_text_color
            editor_line_number_color: Color32::from_gray(150),

            opcode_color: Color32::from_rgb(190, 30, 30),
            help_operand_color: Color32::from_rgb(30, 90, 170),
            help_immediate_color: Color32::from_rgb(20, 130, 20),
            help_offset_color: Color32::from_rgb(90, 50, 170),
            help_binary_layout_fixed_bits_color: Color32::from_gray(120),
            help_pseudo_code_color: Color32::from_gray(70),
            help_title_color: Color32::BLACK,
            help_heading_color: Color32::from_gray(20),
            help_sub_heading_color: Color32::from_gray(50),
            help_strong_label_color: Color32::from_gray(15),
            help_italic_label_color: Color32::from_gray(90),
            help_monospace_color: Color32::from_gray(40),
            help_code_block_text_color: Color32::from_gray(30),
            help_collapsible_header_bg_color: Color32::from_gray(230),
            help_collapsible_header_text_color: Color32::from_gray(20),
            help_info_list_icon_color: Color32::from_rgb(0, 120, 220),
            help_link_color: Color32::from_rgb(0, 110, 230),

            item_spacing: Vec2::new(8.0, 7.0),
            widget_rounding: CornerRadius::same(3),
            window_rounding: CornerRadius::same(4),
            scroll_bar_width: 12.0,
            window_stroke: Stroke::new(1.0, Color32::from_gray(200)),
            panel_stroke: Stroke::new(1.0, Color32::from_gray(210)),
            separator_color: Color32::from_gray(200),
            tooltip_bg_color: Color32::from_gray(240),
            tooltip_text_color: Color32::from_gray(20),
            scrollbar_bg_color: Color32::from_gray(230),
            scrollbar_handle_color: Color32::from_gray(180),
            scrollbar_handle_hovered_color: Color32::from_gray(165),
            scrollbar_handle_active_color: Color32::from_gray(150),
            scrollbar_rounding: CornerRadius::same(2),
            button_padding: Vec2::new(10.0, 6.0),
            indent_width: 20.0,
        }
    }

    pub fn apply_to_style(&self, style: &mut Style) {
        // Base visuals from egui (light/dark)
        style.visuals = match self.base_theme {
            BaseThemeChoice::Light => Visuals::light(),
            BaseThemeChoice::Dark => Visuals::dark(),
        };

        // --- Texts ---
        style.visuals.override_text_color = Some(self.primary_text_color);
        style.visuals.hyperlink_color = self.hyperlink_color;
        // For non-interactive labels, TextEdit, etc.
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.primary_text_color);

        // --- Backgrounds ---
        style.visuals.window_fill = self.window_background;
        style.visuals.widgets.noninteractive.bg_fill = self.panel_background; // For Frames, Groups
        style.visuals.code_bg_color = self.code_bg_color;
        // --- Widget States ---
        // Inactive (interactive, but not hovered/clicked, e.g., a button)
        style.visuals.widgets.inactive.fg_stroke = self.fg_widget_stroke_interactive;
        style.visuals.widgets.inactive.bg_fill = self.widget_fill_interactive;
        style.visuals.widgets.inactive.weak_bg_fill = self.widget_fill_interactive;
        style.visuals.widgets.inactive.bg_stroke = self.bg_widget_stroke_interactive;
        style.visuals.widgets.inactive.corner_radius = self.widget_rounding;

        // Hovered
        style.visuals.widgets.hovered.fg_stroke = self.fg_widget_stroke_hovered;
        style.visuals.widgets.hovered.bg_fill = self.widget_fill_hovered;
        style.visuals.widgets.hovered.weak_bg_fill = self.widget_fill_hovered;
        style.visuals.widgets.hovered.bg_stroke = self.bg_widget_stroke_hovered;
        style.visuals.widgets.hovered.corner_radius = self.widget_rounding;
        style.visuals.widgets.hovered.expansion = 0.5; // No expansion on hover by default

        // Active (clicked/dragged)
        style.visuals.widgets.active.fg_stroke = self.fg_widget_stroke_active;
        style.visuals.widgets.active.bg_fill = self.widget_fill_active;
        style.visuals.widgets.active.weak_bg_fill = self.widget_fill_active;
        style.visuals.widgets.active.bg_stroke = self.bg_widget_stroke_active;
        style.visuals.widgets.active.corner_radius = self.widget_rounding;
        style.visuals.widgets.active.expansion = 1.0; // No expansion on active by default

        let mut disabled_widget_style = style.visuals.widgets.inactive; // Start from inactive
        disabled_widget_style.fg_stroke = self.fg_widget_stroke_disabled;
        disabled_widget_style.bg_fill = self.widget_fill_disabled;
        disabled_widget_style.bg_stroke = self.bg_widget_stroke_disabled;
        // style.visuals.widgets.disabled = disabled_widget_style; // If egui adds this field

        // Open (e.g., a combo box that is open)
        style.visuals.widgets.open = style.visuals.widgets.active; // Often same as active

        // Selection (e.g., for text selection in TextEdit or selected item in list)
        style.visuals.selection.bg_fill = self.accent_color_primary.linear_multiply(0.6);
        style.visuals.selection.stroke = self.selection_stroke;

        // --- Look & Feel ---
        style.spacing.item_spacing = self.item_spacing;
        style.visuals.window_corner_radius = self.window_rounding;
        style.visuals.window_stroke = self.window_stroke;
        style.visuals.widgets.noninteractive.bg_stroke = self.panel_stroke; // Stroke for Frames, Groups

        style.spacing.scroll.bar_width = self.scroll_bar_width;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.visuals.widgets.inactive.bg_fill = self.scrollbar_handle_color; // Scrollbar handle color (using inactive as a proxy)
                                                                              // For more detailed scrollbar theming, egui might need more fields or custom drawing.
                                                                              // The current scrollbar theming in egui is somewhat limited via Style.
                                                                              // These are best guesses for influencing scrollbars:
        style.visuals.widgets.hovered.bg_fill = self.scrollbar_handle_hovered_color; // Potentially for hovered handle
        style.visuals.widgets.active.bg_fill = self.scrollbar_handle_active_color; // Potentially for active handle

        style.spacing.slider_width = 150.0;
        style.spacing.text_edit_width = 200.0;
        style.spacing.button_padding = self.button_padding;
        style.spacing.indent = self.indent_width;
        style.spacing.combo_width = 150.0;
        style.spacing.menu_width = 150.0;

        // Clip rectangle rounding (usually less than window rounding)
        style.visuals.clip_rect_margin = 3.0;
        style.visuals.menu_corner_radius = self.widget_rounding;

        // Separators
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.separator_color); // For horizontal/vertical lines

        // Make sure collapsing headers use panel background and stroke by default
        style.visuals.collapsing_header_frame = true; // Ensure frame is drawn

        // Interaction settings
        style.interaction.tooltip_delay = 0.3; // seconds
        style.interaction.show_tooltips_only_when_still = true;
        style.interaction.selectable_labels = true; // Allow selecting text in labels
        style.interaction.multi_widget_text_select = true;

        // Animation (can be disabled for a "snappier" feel)
        // style.animation_time = 0.0; // Disable animations

        // Example: customizing text styles (fonts would be set globally on context)
        // This requires FontId and TextStyle imports if used.
        // For now, we rely on egui's default font setup.
        // style.text_styles.insert(
        //     egui::TextStyle::Heading,
        //     egui::FontId::proportional(22.0),
        // );
        // style.text_styles.insert(
        //     egui::TextStyle::Body,
        //     egui::FontId::proportional(15.0),
        // );
        // style.text_styles.insert(
        //     egui::TextStyle::Monospace,
        //     egui::FontId::monospace(14.0),
        // );
        // style.text_styles.insert(
        //     egui::TextStyle::Button,
        //     egui::FontId::proportional(15.0),
        // );
        // style.text_styles.insert(
        //     egui::TextStyle::Small,
        //     egui::FontId::proportional(12.0),
        // );
    }

    /// Saves the current theme settings to a .ron file in the "assets" directory.
    /// This function only operates in debug builds and not on wasm targets.
    pub fn save_to_ron_file(&self) {
        #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
        {
            let filename = match self.base_theme {
                BaseThemeChoice::Light => "assets/themes/light.ron",
                BaseThemeChoice::Dark => "assets/themes/dark.ron",
            };

            // Ensure assets directory exists
            if let Err(e) = fs::create_dir_all("assets") {
                eprintln!("[Theme] Failed to create assets directory: {e}");
                return;
            }
            if let Err(e) = fs::create_dir_all("assets/themes") {
                eprintln!("[Theme] Failed to create themes directory: {e}");
                return;
            }

            match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
                Ok(ron_string) => match fs::write(filename, ron_string) {
                    Ok(_) => println!("[Theme] Successfully saved theme to {filename}"),
                    Err(e) => eprintln!("[Theme] Failed to write theme to {filename}: {e}"),
                },
                Err(e) => {
                    eprintln!("[Theme] Failed to serialize theme to RON: {e}");
                }
            }
        }
        // Implicitly does nothing if not (non-wasm and debug build)
    }

    pub fn set_global_theme(&mut self, choice: BaseThemeChoice, ctx: Option<&egui::Context>) {
        *self = match choice {
            BaseThemeChoice::Light => ThemeSettings::light_default(),
            BaseThemeChoice::Dark => ThemeSettings::dark_default(),
        };

        if let Some(ctx) = ctx {
            let mut style = (*ctx.style()).clone();
            self.apply_to_style(&mut style);
            ctx.set_style(style);
            ctx.request_repaint(); // Ensure UI updates immediately
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self::dark_default() // Default to dark theme
    }
}

// lazy_static! {
//     pub static ref CURRENT_THEME_SETTINGS: Mutex<ThemeSettings> =
//         Mutex::new(ThemeSettings::default());
// }
