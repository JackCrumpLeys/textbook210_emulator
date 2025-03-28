use crate::app::{base_to_base, EMULATOR};
use crate::emulator::{CpuState, Emulator, EmulatorCell};
use crate::panes::{Pane, PaneDisplay, PaneTree};
use egui::RichText;
use serde::{Deserialize, Serialize};

use super::{editor::COMPILATION_ARTIFACTS, EmulatorPane};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CpuStatePane {}

impl PaneDisplay for CpuStatePane {
    fn render(&mut self, ui: &mut egui::Ui) {
        let mut emulator = EMULATOR.lock().unwrap();

        // Flags view
        ui.collapsing("Flags", |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(emulator.n.get() == 1, "N")
                    .on_hover_text("Negative Flag")
                    .clicked()
                {
                    emulator.n.set(1);
                    emulator.z.set(0);
                    emulator.p.set(0);
                }

                if ui
                    .selectable_label(emulator.z.get() == 1, "Z")
                    .on_hover_text("Zero Flag")
                    .clicked()
                {
                    emulator.z.set(1);
                    emulator.n.set(0);
                    emulator.p.set(0);
                }

                if ui
                    .selectable_label(emulator.p.get() == 1, "P")
                    .on_hover_text("Positive Flag")
                    .clicked()
                {
                    emulator.p.set(1);
                    emulator.n.set(0);
                    emulator.z.set(0);
                }
            });
        });

        // Processor Cycle view
        ui.collapsing("Processor Cycle", |ui| {
            self.render_cycle_view(ui, &mut *emulator);
        });
    }

    fn title(&self) -> String {
        "CPU State".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "CPU State".to_string(),
            Pane::EmulatorPanes(Box::new(EmulatorPane::Cpu(CpuStatePane::default()))),
        )
    }
}

impl CpuStatePane {
    fn render_cycle_view(&mut self, ui: &mut egui::Ui, emulator: &mut Emulator) {
        let cycles = ["Fetch", "Decode", "ReadMemory", "Execute"];
        let current_cycle = emulator.cpu_state as usize;
        let artifacts = COMPILATION_ARTIFACTS.lock().unwrap();

        let instruction_text = match emulator.ir.get() >> 12 {
            0x1 => "ADD",
            0x5 => "AND",
            0x0 => "BR",
            0xC => "JMP/RET",
            0x4 => "JSR/JSRR",
            0x2 => "LD",
            0xA => "LDI",
            0x6 => "LDR",
            0xE => "LEA",
            0x9 => "NOT",
            0x8 => "RTI",
            0x3 => "ST",
            0xB => "STI",
            0x7 => "STR",
            0xF => "TRAP",
            _ => "Unknown",
        };

        // Find the corresponding source line if available
        // Find the line number associated with the current PC
        let current_pc = emulator.pc.get() as usize;
        let source_line_num = artifacts
            .line_to_address
            .iter()
            .find_map(|(line_num, &addr)| {
                if addr == current_pc {
                    Some(*line_num)
                } else {
                    None
                }
            });

        let source_line_text = source_line_num
            .and_then(|num| artifacts.last_compiled_source.lines().nth(num))
            .map(|s| s.trim())
            .unwrap_or("Unknown instruction");

        let mut description = RichText::new("NO CURRENT CYCLE");

        // Display cycle information
        for (i, cycle) in cycles.iter().enumerate() {
            if i == current_cycle {
                ui.label(
                    RichText::new(format!("-> {}", cycle))
                        .strong()
                        .color(egui::Color32::GREEN),
                );

                // Provide specific description based on the current cycle
                description = match emulator.cpu_state {
                    CpuState::Fetch => RichText::new(format!(
                        "FETCH: Fetching instruction at PC={:#06x}. MAR set to PC. MDR will load from memory. PC incremented.",
                        emulator.pc.get()
                    )).color(egui::Color32::LIGHT_GREEN),

                    CpuState::Decode => RichText::new(format!(
                        "DECODE: Analyzing instruction IR={:#06x} (Opcode: {:X} -> {}). Identifying registers/operands.",
                        emulator.ir.get(), emulator.ir.get() >> 12, instruction_text
                    )).color(egui::Color32::LIGHT_YELLOW),

                    CpuState::ReadMemory => RichText::new(format!(
                        "MEMORY ACCESS: Accessing memory. MAR={:#06x}. MDR will load if read. Relevant for LD, LDI, ST, STI, STR.",
                        emulator.mar.get()
                    )).color(egui::Color32::LIGHT_BLUE),

                    CpuState::Execute => RichText::new(format!(
                        "EXECUTE: Performing operation for IR={:#06x} ('{}', from line: '{}'). May update registers/flags (N={}, Z={}, P={}).",
                        emulator.ir.get(), instruction_text, source_line_text,
                        emulator.n.get(), emulator.z.get(), emulator.p.get()
                    )).color(egui::Color32::GOLD),
                };
            } else {
                ui.label(RichText::new(format!("  {}", cycle)).color(egui::Color32::GRAY));
            }
        }
        ui.label(description);
        ui.separator();

        // Display relevant registers/flags for the current state
        ui.horizontal(|ui| {
            ui.label("Flags:");
            let n_text = format!("N={}", emulator.n.get());
            let z_text = format!("Z={}", emulator.z.get());
            let p_text = format!("P={}", emulator.p.get());
            let flag_color = if emulator.cpu_state == CpuState::Execute {
                egui::Color32::LIGHT_GREEN
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(n_text).color(flag_color));
            ui.label(RichText::new(z_text).color(flag_color));
            ui.label(RichText::new(p_text).color(flag_color));
        });

        ui.horizontal(|ui| {
            ui.label("Memory:");
            let mar_text = format!("MAR={:#06x}", emulator.mar.get());
            let mdr_text = format!("MDR={:#06x}", emulator.mdr.get());
            let mem_color = if emulator.cpu_state == CpuState::Fetch
                || emulator.cpu_state == CpuState::ReadMemory
            {
                egui::Color32::YELLOW
            } else if emulator.cpu_state == CpuState::Execute {
                // Highlight potential write in execute if applicable (though write happens after)
                egui::Color32::LIGHT_GREEN
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(mar_text).color(mem_color));
            ui.label(RichText::new(mdr_text).color(mem_color));
        });

        ui.horizontal(|ui| {
            ui.label("Control:");
            let ir_text = format!("IR={:#06x}", emulator.ir.get());
            let pc_text = format!("PC={:#06x}", emulator.pc.get());
            let ir_color = if emulator.cpu_state == CpuState::Decode {
                egui::Color32::LIGHT_GREEN
            } else if emulator.cpu_state == CpuState::Fetch {
                egui::Color32::YELLOW // IR will be loaded next
            } else {
                ui.visuals().text_color()
            };
            let pc_color = if emulator.cpu_state == CpuState::Fetch {
                egui::Color32::LIGHT_GREEN // PC is being used
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(ir_text).color(ir_color));
            ui.label(RichText::new(pc_text).color(pc_color));
        });
    }
}
