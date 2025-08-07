use crate::emulator::Emulator;
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use crate::theme::ThemeSettings;
use egui::{Key, OutputCommand, RichText};
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct IoPane {
    terminal_input: String,
    interactive_input: String,
}

impl PaneDisplay for IoPane {
    fn render(&mut self, ui: &mut egui::Ui, emulator: &mut Emulator, _theme: &mut ThemeSettings) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Display terminal output
            ui.label(RichText::new("Terminal:").strong());

            let terminal_height = 200.0;
            egui::ScrollArea::vertical()
                .max_height(terminal_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut emulator.output)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .interactive(false)
                            .font(egui::TextStyle::Monospace),
                    );
                });

            ui.horizontal(|ui| {
                ui.label(">");
                ui.add(
                    egui::TextEdit::singleline(&mut self.terminal_input)
                        .desired_width(ui.available_width())
                        .hint_text("Type here to send input to the emulator...")
                        .font(egui::TextStyle::Monospace),
                );

                if self.terminal_input.len() >= 1 {
                    emulator.set_in_char(self.terminal_input.chars().last().unwrap());
                    self.terminal_input.clear();
                }
            });

            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("Clear Output").clicked() {
                    emulator.output.clear();
                }

                let output_clone = emulator.output.clone();
                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| o.commands.push(OutputCommand::CopyText(output_clone)));
                }
            });
        });
    }

    fn title(&self) -> String {
        "Terminal".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Terminal".to_string(),
            Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Output(
                IoPane::default(),
            )))),
        )
    }
}
