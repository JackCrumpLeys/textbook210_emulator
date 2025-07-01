use crate::app::EMULATOR;
use crate::emulator::parse::ParseOutput;
use crate::emulator::Emulator;
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use egui::RichText;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Mutex;

use super::EmulatorPane;

lazy_static! {
    pub static ref COMPILATION_ARTIFACTS: Mutex<CompilationArtifacts> =
        Mutex::new(CompilationArtifacts::default());
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompilationArtifacts {
    pub last_compiled_source: String,
    pub line_to_address: HashMap<usize, usize>,
    pub labels: HashMap<String, u16>,
    pub addr_to_label: HashMap<u16, String>, // to optimise when fetching lable from addr
    pub orig_address: u16,
    pub error: Option<(String, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EditorPane {
    program: String,
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
        }
    }
}

impl PaneDisplay for EditorPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_code_editor::CodeEditor::default()
                .with_syntax(
                    egui_code_editor::Syntax::new("lc3_assembly")
                        .with_comment(";")
                        .with_keywords(BTreeSet::from([
                            "ADD", "AND", "BR", "BRN", "BRZ", "BRP", "BRNZ", "BRNP", "BRZP",
                            "BRNZP", "JMP", "JSR", "JSRR", "LD", "LDI", "LDR", "LEA", "NOT", "RET",
                            "RTI", "ST", "STI", "STR", "TRAP", "GETC", "OUT", "PUTS", "IN", "HALT",
                        ]))
                        .with_special(BTreeSet::from([
                            ":", ".ORIG", ".FILL", ".BLKW", ".STRINGZ", ".END",
                        ]))
                        .with_case_sensitive(false),
                )
                .vscroll(false)
                .with_theme(egui_code_editor::ColorTheme::SONOKAI)
                .show(ui, &mut self.program);

            let mut artifacts = COMPILATION_ARTIFACTS.lock().unwrap();

            if let Some((error, line)) = &artifacts.error {
                ui.label(
                    RichText::new(format!("Error on line {line}: {error}"))
                        .small()
                        .color(ui.visuals().warn_fg_color),
                );
            }

            ui.horizontal(|ui| {
                if ui.button("Reset & Compile").clicked() {
                    let data_to_load = Emulator::parse_program(&self.program);
                    let mut emulator = EMULATOR.lock().unwrap();
                    *emulator = Emulator::new(); // Reset emulator state

                    if let Ok(ParseOutput {
                        machine_code,
                        line_to_address,
                        labels,
                        orig_address,
                    }) = data_to_load
                    {
                        artifacts.line_to_address = line_to_address;
                        artifacts.labels = labels.clone();
                        artifacts.addr_to_label = labels.into_iter().map(|(x, y)| (y, x)).collect();
                        artifacts.orig_address = orig_address;
                        artifacts.error = None;
                        artifacts.last_compiled_source = self.program.clone();

                        // Flash memory
                        emulator.flash_memory(machine_code, orig_address);
                    } else {
                        artifacts.error = Some(data_to_load.unwrap_err());
                        artifacts.line_to_address.clear();
                        artifacts.labels.clear();
                        artifacts.addr_to_label.clear();
                        artifacts.last_compiled_source.clear();
                    }
                }
            });
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
