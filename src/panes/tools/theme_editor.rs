use egui::{Color32, CornerRadius, RichText, ScrollArea, Stroke, Theme, Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    panes::{Pane, PaneDisplay, PaneTree, RealPane},
    theme::{BaseThemeChoice, ThemeSettings, CURRENT_THEME_SETTINGS},
};

use super::ToolPanes;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeEditorPane {
    // Holds a temporary copy of settings for editing, applied on change
    live_settings: ThemeSettings,
    // To manage which base theme preset is selected for reset/save purposes
    selected_base_for_file_op: BaseThemeChoice,
}

impl Default for ThemeEditorPane {
    fn default() -> Self {
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();
        Self {
            live_settings: theme_settings.clone(),
            selected_base_for_file_op: theme_settings.base_theme,
        }
    }
}

impl ThemeEditorPane {
    fn apply_live_settings_to_global(&self, ctx: &egui::Context) {
        let mut global_settings = CURRENT_THEME_SETTINGS.lock().unwrap();
        *global_settings = self.live_settings.clone();

        ctx.set_theme(match global_settings.base_theme {
            BaseThemeChoice::Light => Theme::Light,
            BaseThemeChoice::Dark => Theme::Dark,
        });

        let mut style = (*ctx.style()).clone();
        global_settings.apply_to_style(&mut style);
        ctx.set_style(style);
        ctx.request_repaint();
    }
}

impl PaneDisplay for ThemeEditorPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        // Refresh live_settings from global if they differ (e.g., theme changed elsewhere)
        // This is a simple way to keep them in sync. More complex logic might be needed
        // if we want to detect "unsaved changes" explicitly.
        let global_base_theme = CURRENT_THEME_SETTINGS.lock().unwrap().base_theme;
        if self.live_settings.base_theme != global_base_theme {
            self.live_settings = CURRENT_THEME_SETTINGS.lock().unwrap().clone();
            self.selected_base_for_file_op = global_base_theme;
        }

        ui.horizontal(|ui| {
            if ui.button("Reset to Dark").clicked() {
                self.live_settings = ThemeSettings::dark_default();
                self.selected_base_for_file_op = BaseThemeChoice::Dark;
                self.apply_live_settings_to_global(ui.ctx());
            }
            if ui.button("Reset to Light").clicked() {
                self.live_settings = ThemeSettings::light_default();
                self.selected_base_for_file_op = BaseThemeChoice::Light;
                self.apply_live_settings_to_global(ui.ctx());
            }

            // Conditional save button
            #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
            {
                ui.menu_button("Save Current to File...", |ui| {
                    if ui
                        .button("Save as Dark Theme (dark.ron)".to_string())
                        .clicked()
                    {
                        let mut settings_to_save = self.live_settings.clone();
                        settings_to_save.base_theme = BaseThemeChoice::Dark;
                        settings_to_save.save_to_ron_file();
                        ui.close();
                    }
                    if ui
                        .button("Save as Light Theme (light.ron)".to_string())
                        .clicked()
                    {
                        let mut settings_to_save = self.live_settings.clone();
                        settings_to_save.base_theme = BaseThemeChoice::Light;
                        settings_to_save.save_to_ron_file();
                        ui.close();
                    }
                });
            }
        });
        ui.separator();

        let mut settings_changed = false;

        egui::SidePanel::left("theme_settings_panel")
            .resizable(true)
            .default_width(ui.available_width() * 0.5)
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Theme Settings");

                    // Pass a mutable reference to self.live_settings and a flag
                    settings_changed |= render_settings_editor(ui, &mut self.live_settings);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Theme Preview");
                render_preview_area(ui, &self.live_settings);
            });
        });

        if settings_changed {
            // If any setting was changed, apply it to the global theme
            self.apply_live_settings_to_global(ui.ctx());
        }
    }

    fn title(&self) -> String {
        "Theme Editor".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Theme Editor".to_string(),
            Pane::new(RealPane::ToolPanes(Box::new(ToolPanes::ThemeEditor(
                Box::default(),
            )))),
        )
    }
}

// Helper functions for rendering specific setting types
// Returns true if any setting was changed

fn render_color_setting(ui: &mut Ui, label: &str, color: &mut Color32) -> bool {
    let mut chd = false;
    ui.horizontal(|ui| {
        ui.label(label);
        chd = ui.color_edit_button_srgba(color).changed()
    });
    chd
}

fn render_stroke_setting(ui: &mut Ui, label: &str, stroke: &mut Stroke) -> bool {
    let mut changed = false;
    ui.group(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.label("Width:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut stroke.width)
                        .speed(0.1)
                        .range(0.0..=10.0),
                )
                .changed();
            ui.label("Color:");
            changed |= ui.color_edit_button_srgba(&mut stroke.color).changed();
        });
    })
    .response
    .changed()
        || changed
}

fn render_vec2_setting(
    ui: &mut Ui,
    label: &str,
    vec: &mut Vec2,
    speed: f32,
    clamp_min: f32,
    clamp_max: f32,
) -> bool {
    let mut changed = false;
    ui.group(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.label("X:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut vec.x)
                        .speed(speed)
                        .range(clamp_min..=clamp_max),
                )
                .changed();
            ui.label("Y:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut vec.y)
                        .speed(speed)
                        .range(clamp_min..=clamp_max),
                )
                .changed();
        });
    })
    .response
    .changed()
        || changed
}

fn render_corner_radius_setting(ui: &mut Ui, label: &str, radius: &mut CornerRadius) -> bool {
    let mut changed = false;
    ui.group(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.label("NW:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut radius.nw)
                        .speed(0.1)
                        .range(0.0..=20.0),
                )
                .changed();
            ui.label("NE:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut radius.ne)
                        .speed(0.1)
                        .range(0.0..=20.0),
                )
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("SW:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut radius.sw)
                        .speed(0.1)
                        .range(0.0..=20.0),
                )
                .changed();
            ui.label("SE:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut radius.se)
                        .speed(0.1)
                        .range(0.0..=20.0),
                )
                .changed();
        });
        if ui.button("Set All Uniform").clicked() {
            let first = radius.nw;
            radius.ne = first;
            radius.sw = first;
            radius.se = first;
            changed = true;
        }
    })
    .response
    .changed()
        || changed
}

fn render_f32_setting(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    speed: f32,
    clamp_min: f32,
    clamp_max: f32,
) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(value)
                .speed(speed)
                .range(clamp_min..=clamp_max),
        )
    })
    .response
    .changed()
}

fn render_base_theme_choice_setting(
    ui: &mut Ui,
    label: &str,
    choice: &mut BaseThemeChoice,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed |= ui
            .radio_value(choice, BaseThemeChoice::Light, "Light")
            .changed();
        changed |= ui
            .radio_value(choice, BaseThemeChoice::Dark, "Dark")
            .changed();
    });
    changed
}

fn render_settings_editor(ui: &mut Ui, settings: &mut ThemeSettings) -> bool {
    let mut changed = false;

    egui::CollapsingHeader::new("Base Theme")
        .default_open(false)
        .show(ui, |ui| {
            changed |= render_base_theme_choice_setting(
                ui,
                "Base Theme Preset:",
                &mut settings.base_theme,
            );
        });

    egui::CollapsingHeader::new("General UI Colors")
        .default_open(false)
        .show(ui, |ui| {
            changed |=
                render_color_setting(ui, "Primary Text Color:", &mut settings.primary_text_color);
            changed |= render_color_setting(
                ui,
                "Secondary Text Color:",
                &mut settings.secondary_text_color,
            );
            changed |=
                render_color_setting(ui, "Strong Text Color:", &mut settings.strong_text_color);
            changed |= render_color_setting(ui, "Hyperlink Color:", &mut settings.hyperlink_color);
            changed |=
                render_color_setting(ui, "Window Background:", &mut settings.window_background);
            changed |=
                render_color_setting(ui, "Panel Background:", &mut settings.panel_background);
            changed |=
                render_color_setting(ui, "Code Background Color:", &mut settings.code_bg_color);
        });

    egui::CollapsingHeader::new("Accent Colors")
        .default_open(false)
        .show(ui, |ui| {
            changed |=
                render_color_setting(ui, "Primary Accent:", &mut settings.accent_color_primary);
            changed |= render_color_setting(
                ui,
                "Secondary Accent:",
                &mut settings.accent_color_secondary,
            );
            changed |=
                render_color_setting(ui, "Tertiary Accent:", &mut settings.accent_color_tertiary);
            changed |=
                render_color_setting(ui, "Positive Accent:", &mut settings.accent_color_positive);
            changed |=
                render_color_setting(ui, "Negative Accent:", &mut settings.accent_color_negative);
        });

    egui::CollapsingHeader::new("Widget States")
        .default_open(false)
        .show(ui, |ui| {
            changed |=
                render_color_setting(ui, "Widget Text Color:", &mut settings.widget_text_color);
            changed |= render_color_setting(
                ui,
                "Fill Interactive:",
                &mut settings.widget_fill_interactive,
            );

            ui.separator();
            changed |= render_color_setting(ui, "Fill Hovered:", &mut settings.widget_fill_hovered);
            changed |= render_color_setting(ui, "Fill Active:", &mut settings.widget_fill_active);

            ui.separator();
            changed |=
                render_color_setting(ui, "Fill Disabled:", &mut settings.widget_fill_disabled);
            changed |= render_stroke_setting(
                ui,
                "Stroke Interactive:",
                &mut settings.bg_widget_stroke_interactive,
            );
            changed |= render_stroke_setting(
                ui,
                "Stroke Hovered:",
                &mut settings.bg_widget_stroke_hovered,
            );
            changed |=
                render_stroke_setting(ui, "Stroke Active:", &mut settings.bg_widget_stroke_active);
            changed |= render_stroke_setting(
                ui,
                "Stroke Disabled:",
                &mut settings.bg_widget_stroke_disabled,
            );

            ui.separator();

            changed |= render_stroke_setting(
                ui,
                "FG Stroke Interactive:",
                &mut settings.fg_widget_stroke_interactive,
            );
            changed |= render_stroke_setting(
                ui,
                "FG Stroke Hovered:",
                &mut settings.fg_widget_stroke_hovered,
            );
            changed |= render_stroke_setting(
                ui,
                "FG Stroke Active:",
                &mut settings.fg_widget_stroke_active,
            );
            changed |= render_stroke_setting(
                ui,
                "FG Stroke Disabled:",
                &mut settings.fg_widget_stroke_disabled,
            );
            ui.separator();

            changed |=
                render_stroke_setting(ui, "Selection Stroke:", &mut settings.selection_stroke);
        });

    egui::CollapsingHeader::new("Semantic Colors")
        .default_open(false)
        .show(ui, |ui| {
            changed |= render_color_setting(ui, "Error FG Color:", &mut settings.error_fg_color);
            changed |= render_color_setting(ui, "Error BG Color:", &mut settings.error_bg_color);
            changed |= render_color_setting(ui, "Warning FG Color:", &mut settings.warn_fg_color);
            changed |= render_color_setting(ui, "Warning BG Color:", &mut settings.warn_bg_color);
            changed |=
                render_color_setting(ui, "Success FG Color:", &mut settings.success_fg_color);
            changed |=
                render_color_setting(ui, "Success BG Color:", &mut settings.success_bg_color);
            changed |= render_color_setting(ui, "Info FG Color:", &mut settings.info_fg_color);
            changed |= render_color_setting(ui, "Info BG Color:", &mut settings.info_bg_color);
        });

    egui::CollapsingHeader::new("Emulator Specific Colors")
        .default_open(false)
        .show(ui, |ui| {
            changed |= render_color_setting(ui, "PC Line BG:", &mut settings.pc_line_bg);
            changed |= render_color_setting(ui, "Breakpoint BG:", &mut settings.breakpoint_bg);
            changed |= render_color_setting(ui, "Highlight BG:", &mut settings.highlight_bg);
            changed |= render_color_setting(
                ui,
                "Memory Address Color:",
                &mut settings.memory_address_color,
            );
            changed |=
                render_color_setting(ui, "Memory Label Color:", &mut settings.memory_label_color);
            changed |=
                render_color_setting(ui, "Memory Value Color:", &mut settings.memory_value_color);
            changed |=
                render_color_setting(ui, "Memory ASCII Color:", &mut settings.memory_ascii_color);
            changed |= render_color_setting(
                ui,
                "Memory Instruction Color:",
                &mut settings.memory_instruction_color,
            );
            changed |= render_color_setting(
                ui,
                "Register Name Color:",
                &mut settings.register_name_color,
            );
            changed |= render_color_setting(
                ui,
                "Register Value Color:",
                &mut settings.register_value_color,
            );
            changed |= render_color_setting(
                ui,
                "CPU State Active Color:",
                &mut settings.cpu_state_active_color,
            );
            changed |= render_color_setting(
                ui,
                "CPU State Inactive Color:",
                &mut settings.cpu_state_inactive_color,
            );
            changed |= render_color_setting(
                ui,
                "CPU State Description Color:",
                &mut settings.cpu_state_description_color,
            );
        });

    egui::CollapsingHeader::new("Help Pane Colors")
        .default_open(false)
        .show(ui, |ui| {
            changed |=
                render_color_setting(ui, "Help Opcode Color:", &mut settings.help_opcode_color);
            changed |=
                render_color_setting(ui, "Help Operand Color:", &mut settings.help_operand_color);
            changed |= render_color_setting(
                ui,
                "Help Immediate Color:",
                &mut settings.help_immediate_color,
            );
            changed |=
                render_color_setting(ui, "Help Offset Color:", &mut settings.help_offset_color);
            changed |= render_color_setting(
                ui,
                "Help Binary Layout Fixed Bits Color:",
                &mut settings.help_binary_layout_fixed_bits_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Pseudo Code Color:",
                &mut settings.help_pseudo_code_color,
            );
            changed |=
                render_color_setting(ui, "Help Title Color:", &mut settings.help_title_color);
            changed |=
                render_color_setting(ui, "Help Heading Color:", &mut settings.help_heading_color);
            changed |= render_color_setting(
                ui,
                "Help Sub Heading Color:",
                &mut settings.help_sub_heading_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Strong Label Color:",
                &mut settings.help_strong_label_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Italic Label Color:",
                &mut settings.help_italic_label_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Monospace Color:",
                &mut settings.help_monospace_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Code Block Text Color:",
                &mut settings.help_code_block_text_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Collapsible Header BG Color:",
                &mut settings.help_collapsible_header_bg_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Collapsible Header Text Color:",
                &mut settings.help_collapsible_header_text_color,
            );
            changed |= render_color_setting(
                ui,
                "Help Info List Icon Color:",
                &mut settings.help_info_list_icon_color,
            );
            changed |= render_color_setting(ui, "Help Link Color:", &mut settings.help_link_color);
        });

    egui::CollapsingHeader::new("Look & Feel")
        .default_open(false)
        .show(ui, |ui| {
            changed |= render_vec2_setting(
                ui,
                "Item Spacing:",
                &mut settings.item_spacing,
                0.1,
                0.0,
                20.0,
            );
            changed |=
                render_corner_radius_setting(ui, "Widget Rounding:", &mut settings.widget_rounding);
            changed |=
                render_corner_radius_setting(ui, "Window Rounding:", &mut settings.window_rounding);
            changed |= render_f32_setting(
                ui,
                "Scroll Bar Width:",
                &mut settings.scroll_bar_width,
                0.1,
                1.0,
                30.0,
            );
            changed |= render_stroke_setting(ui, "Window Stroke:", &mut settings.window_stroke);
            changed |= render_stroke_setting(ui, "Panel Stroke:", &mut settings.panel_stroke);
            changed |= render_color_setting(ui, "Separator Color:", &mut settings.separator_color);
            changed |=
                render_color_setting(ui, "Tooltip BG Color:", &mut settings.tooltip_bg_color);
            changed |=
                render_color_setting(ui, "Tooltip Text Color:", &mut settings.tooltip_text_color);
            changed |=
                render_color_setting(ui, "Scrollbar BG Color:", &mut settings.scrollbar_bg_color);
            changed |= render_color_setting(
                ui,
                "Scrollbar Handle Color:",
                &mut settings.scrollbar_handle_color,
            );
            changed |= render_color_setting(
                ui,
                "Scrollbar Handle Hovered Color:",
                &mut settings.scrollbar_handle_hovered_color,
            );
            changed |= render_color_setting(
                ui,
                "Scrollbar Handle Active Color:",
                &mut settings.scrollbar_handle_active_color,
            );
            changed |= render_corner_radius_setting(
                ui,
                "Scrollbar Rounding:",
                &mut settings.scrollbar_rounding,
            );
            changed |= render_vec2_setting(
                ui,
                "Button Padding:",
                &mut settings.button_padding,
                0.1,
                0.0,
                20.0,
            );
            changed |= render_f32_setting(
                ui,
                "Indent Width:",
                &mut settings.indent_width,
                0.1,
                0.0,
                50.0,
            );
        });

    changed
}

fn render_preview_area(ui: &mut Ui, _settings: &ThemeSettings) {
    // This function will render various egui widgets to preview the theme.
    // It uses the currently applied theme from ui.style() implicitly.

    ui.group(|ui| {
        ui.label("This is a normal label using primary_text_color.");
        ui.label(
            RichText::new("This is a secondary_text_color label.")
                .color(ui.style().visuals.text_color().gamma_multiply(0.7)),
        ); // Approximation
        ui.label(RichText::new("This is strong_text_color.").strong());
        ui.hyperlink_to("This is a hyperlink_color", "https://example.com");
    });

    ui.group(|ui| {
        ui.heading("Widget Preview");
        ui.button("Normal Button").clicked();
        let mut checkbox_val = true;
        ui.checkbox(&mut checkbox_val, "Checkbox");
        let mut radio_val = 0;
        ui.radio_value(&mut radio_val, 0, "Radio 1");
        ui.radio_value(&mut radio_val, 1, "Radio 2");
        let mut slider_val = 50.0;
        ui.add(egui::Slider::new(&mut slider_val, 0.0..=100.0).text("Slider"));
        let mut text_edit_val = "Text Edit".to_string();
        ui.text_edit_singleline(&mut text_edit_val);

        ui.add_enabled(false, egui::Button::new("Disabled Button"));
    });

    ui.group(|ui| {
        ui.heading("Semantic Colors Preview");
        ui.label(
            RichText::new("This is an error_fg_color message.")
                .color(ui.style().visuals.error_fg_color),
        );
        ui.label(
            RichText::new("This is a warn_fg_color message.")
                .color(ui.style().visuals.warn_fg_color),
        );
        ui.label(
            RichText::new("This is an info_fg_color message.")
                .color(ui.style().visuals.text_color()),
        ); // Assuming info uses primary
    });

    ui.group(|ui| {
        ui.heading("Code Block Preview");
        egui::Frame::canvas(ui.style())
            .fill(ui.style().visuals.code_bg_color)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(".ORIG x3000\nADD R1, R2, #5\nHALT")
                        .monospace()
                        .color(
                            ui.style()
                                .visuals
                                .override_text_color
                                .unwrap_or(ui.style().visuals.text_color()),
                        ),
                );
            });
    });

    egui::CollapsingHeader::new("Collapsing Header").show(ui, |ui| {
        ui.label("Content inside a collapsing header.");
    });

    ui.separator();

    ui.label("Tooltip Preview (hover button):");
    ui.button("Hover Me").on_hover_text("This is a tooltip!");
}
