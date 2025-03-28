use egui::RichText;
use serde::{Deserialize, Serialize};

use crate::app::EMULATOR;
use crate::{
    app::base_to_base,
    panes::{Pane, PaneDisplay, PaneTree},
};

use super::ToolPanes;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseConverter {
    input: String,
    output_hist: Vec<String>,
    alphabet: String,
    base_in: u32,
    base_out: u32,
    case_sensitive: bool,
    uppercase: bool,
}

impl PaneDisplay for BaseConverter {
    fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("This is a base converter. Enter a number, select the input and output bases, adjust the alphabet, and click 'Convert' to see the result. You can also toggle case sensitivity and choose between uppercase and lowercase conversion.");

        if self.case_sensitive {
            ui.label(RichText::new("⚠ Note: Case sensitivity is enabled. ⚠")
                            .small()
                            .color(ui.visuals().warn_fg_color)).on_hover_text("Case sensitivity is enabled.  You can change this behavior by toggling the 'Case Sensitive' checkbox.");
        }
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.input);
            ui.label("->");
            if let Some(most_recent_output) = self.output_hist.last() {
                ui.label(most_recent_output);
            } else {
                ui.label("");
            }
            if ui.button("Convert").clicked() {
                // Call the stub function base_to_base
                if !self.case_sensitive {
                    if self.uppercase {
                        self.input = self.input.to_uppercase();
                    } else {
                        self.input = self.input.to_lowercase();
                    }
                }
                let output = base_to_base(self.base_in, self.base_out, &self.input, &self.alphabet);
                self.output_hist.push(output);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Alphabet:");
            ui.text_edit_singleline(&mut self.alphabet);
        });

        let max_base = self.alphabet.len() as u32;

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.base_in, 2..=max_base));
            ui.add(egui::Slider::new(&mut self.base_out, 2..=max_base));
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.case_sensitive, "Case Sensitive");
            if !self.case_sensitive {
                ui.checkbox(&mut self.uppercase, "Uppercase");
            }
        });

        ui.separator();

        egui::CollapsingHeader::new("History")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in self.output_hist.iter() {
                        ui.label(line);
                    }
                });
            });
    }

    fn title(&self) -> String {
        "Base Converter".to_owned()
    }

    fn children() -> PaneTree {
        PaneTree::Children(
            "Tools".to_string(),
            vec![PaneTree::Pane(
                "Base Converter".to_string(),
                Pane::ToolPanes(Box::new(ToolPanes::BaseConverter(Self::default()))),
            )],
        )
    }
}
