use crate::emulator::parse::{ParseError, ParseOutput};
use crate::emulator::Emulator;
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use crate::theme::ThemeSettings;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EditorPane {
    program: String,
    fade: f32,
    last_compilation_was_successful: bool,
}

impl Default for EditorPane {
    fn default() -> Self {
        Self {
            program: r#".ORIG x3000
; Simple Hello World program
LEA R0, MESSAGE    ; Load the address of the message
PUTS               ; Output the string
HALT               ; Halt the program

MESSAGE: .STRINGZ "Hello, World!"
.END"#
                .to_string(),
            fade: 0.0,
            last_compilation_was_successful: false,
        }
    }
}

impl PaneDisplay for EditorPane {
    fn render(&mut self, ui: &mut egui::Ui, emulator: &mut Emulator, theme: &mut ThemeSettings) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Make the code editor borderless and fill the available width
            let editor_frame = egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin::same(0));

            editor_frame.show(ui, |ui| {
                egui_code_editor::CodeEditor::default()
                    .with_syntax(
                        egui_code_editor::Syntax::new("lc3_assembly")
                            .with_comment(";")
                            .with_keywords(BTreeSet::from([
                                "ADD", "AND", "BR", "BRN", "BRZ", "BRP", "BRNZ", "BRNP", "BRZP",
                                "BRNZP", "JMP", "JSR", "JSRR", "LD", "LDI", "LDR", "LEA", "NOT",
                                "RET", "RTI", "ST", "STI", "STR", "TRAP", "GETC", "OUT", "PUTS",
                                "IN", "HALT",
                            ]))
                            .with_special(BTreeSet::from([
                                ":", ".ORIG", ".FILL", ".BLKW", ".STRINGZ", ".END",
                            ]))
                            .with_case_sensitive(false),
                    )
                    .vscroll(false)
                    .with_theme(egui_code_editor::ColorTheme::SONOKAI)
                    .show(ui, &mut self.program);
            });

            // Show error or success feedback
            {
                let artifacts = &mut emulator.metadata;
                if let Some(error) = &artifacts.error {
                    match error {
                        ParseError::TokenizeError(s, l) => {
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                format!("Syntax error on line {l}: {s}"),
                            );
                        }
                        ParseError::GenerationError(s, token_span) => {
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                format!("Code generation error at {token_span:?}: {s}"),
                            );
                        }
                    }
                } else if !artifacts.last_compiled_source.is_empty()
                    && self.last_compilation_was_successful
                {
                    // Only show success if there is a compiled source and no error
                    ui.colored_label(
                        theme.success_fg_color,
                        egui::RichText::new("Compiled successfully!").strong(),
                    );
                }
            }
            ui.add_space(8.0);

            // Blend between green and gray based on self.fade
            let just_compiled = match self.last_compilation_was_successful {
                true => theme.accent_color_positive,
                false => theme.accent_color_negative,
            };
            let base = theme.accent_color_primary;
            let fade = self.fade.clamp(0.0, 1.0);

            let blend = |a: egui::Color32, b: egui::Color32, t: f32| -> egui::Color32 {
                let t = t.clamp(0.0, 1.0);
                let r = (a.r() as f32 * t + b.r() as f32 * (1.0 - t)) as u8;
                let g = (a.g() as f32 * t + b.g() as f32 * (1.0 - t)) as u8;
                let b_ = (a.b() as f32 * t + b.b() as f32 * (1.0 - t)) as u8;
                let a_ = (a.a() as f32 * t + b.a() as f32 * (1.0 - t)) as u8;
                egui::Color32::from_rgba_premultiplied(r, g, b_, a_)
            };

            let button_color = blend(just_compiled, base, fade);

            ui.horizontal(|ui| {
                let button = egui::Button::new("Compile").fill(button_color);
                if ui.add(button).clicked() {
                    let data_to_load =
                        Emulator::parse_program(&self.program, Some(&mut emulator.metadata));
                    if let Ok(ParseOutput {
                        machine_code,
                        orig_address,
                        ..
                    }) = data_to_load
                    {
                        // Flash memory
                        emulator.flash_memory(machine_code, orig_address);

                        self.fade = 1.0;
                        self.last_compilation_was_successful = true;
                    } else {
                        self.fade = 1.0;
                        self.last_compilation_was_successful = false;
                    }
                }
            });

            // Decrease fade every tick
            if self.fade > 0.0 {
                self.fade = (self.fade - 0.04).max(0.0);
            }
        });
    }

    fn title(&self) -> String {
        "Editor".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Editor".to_string(),
            Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Editor(
                EditorPane::default(),
            )))),
        )
    }
}
