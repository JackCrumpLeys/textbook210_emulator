use crate::app::EMULATOR;
use crate::emulator::{CpuState, Emulator};
use crate::panes::{Pane, PaneDisplay, PaneTree};
use egui::RichText;
use serde::{Deserialize, Serialize};

use super::{editor::COMPILATION_ARTIFACTS, EmulatorPane};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CpuStatePane {}

impl PaneDisplay for CpuStatePane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
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
                self.render_cycle_view(ui, &mut emulator);
            });
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
        let cycles = [
            "Fetch",
            "Decode",
            "Evaluate Address",
            "Fetch Operands",
            "Execute Operation",
            "Store Result",
        ];
        let current_cycle_index = match emulator.cpu_state {
            CpuState::Fetch => 0,
            CpuState::Decode => 1,
            CpuState::EvaluateAddress(_) => 2,
            CpuState::FetchOperands(_) => 3,
            CpuState::ExecuteOperation(_) => 4,
            CpuState::StoreResult(_) => 5,
        };

        let artifacts = COMPILATION_ARTIFACTS.lock().unwrap();

        // Determine instruction text based on IR, even if state is Fetch/Decode
        let instruction_text = if !matches!(emulator.cpu_state, CpuState::Fetch) {
            match emulator.ir.get() >> 12 {
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
            }
        } else {
            "Loading..." // Or some placeholder while fetching
        };

        // Find the corresponding source line if available (based on PC before fetch completes)
        let pc_for_source_lookup = if matches!(emulator.cpu_state, CpuState::Fetch) {
            emulator.pc.get().wrapping_sub(0) // Show source for the instruction *being* fetched
        } else {
            // For other states, the instruction is already fetched, PC points to the *next* instruction.
            // So, look up the source for PC-1.
            emulator.pc.get().wrapping_sub(1)
        };

        let source_line_num = artifacts
            .line_to_address
            .iter()
            .find_map(|(line_num, &addr)| {
                if addr == pc_for_source_lookup as usize {
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
        for (i, cycle_name) in cycles.iter().enumerate() {
            if i == current_cycle_index {
                ui.label(
                    RichText::new(format!("-> {}", cycle_name))
                        .strong()
                        .color(egui::Color32::GREEN),
                );

                // Provide specific description based on the current cycle
                description = match &emulator.cpu_state {
                    CpuState::Fetch => RichText::new(format!(
                        "FETCH: Reading instruction from Mem[PC={:#06x}] into IR. Incrementing PC.",
                        emulator.pc.get() // PC holds the *address* being fetched now
                    )).color(egui::Color32::LIGHT_GREEN),

                    CpuState::Decode => RichText::new(format!(
                        "DECODE: Analyzing instruction IR={:#06x} (Opcode: {:X} -> {}). Preparing for next phase.",
                        emulator.ir.get(), emulator.ir.get() >> 12, instruction_text
                    )).color(egui::Color32::LIGHT_YELLOW),

                    CpuState::EvaluateAddress(op) => RichText::new(format!(
                        "EVAL ADDR: Calculating memory/target address for {}. MAR may be set.",
                        op
                    )).color(egui::Color32::from_rgb(65, 105, 225)),

                    CpuState::FetchOperands(op) => RichText::new(format!(
                        "FETCH OPS: Reading operands for {} from registers or memory (via MAR={:#06x}). MDR may load.",
                        op,
                        emulator.mar.get()
                    )).color(egui::Color32::LIGHT_BLUE),

                    CpuState::ExecuteOperation(op) => RichText::new(format!(
                        "EXECUTE: Performing operation for {} ('{}', from line: '{}'). ALU computes, PC may change. Flags (N={}, Z={}, P={}) may update.",
                        op,
                         instruction_text, source_line_text,
                        emulator.n.get(), emulator.z.get(), emulator.p.get()
                    )).color(egui::Color32::GOLD),

                    CpuState::StoreResult(op) => RichText::new(format!(
                         "STORE RESULT: Writing result for {} to register or setting up memory write (MAR={:#06x}, MDR={:#06x}). Flags may update.",
                         op,
                        emulator.mar.get(), emulator.mdr.get()
                    )).color(egui::Color32::LIGHT_RED),
                };
            } else {
                ui.label(RichText::new(format!("  {}", cycle_name)).color(egui::Color32::GRAY));
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
            // Flags might be updated in Execute or StoreResult
            let flag_color = if matches!(emulator.cpu_state, CpuState::ExecuteOperation(_))
                || matches!(emulator.cpu_state, CpuState::StoreResult(_))
            {
                if emulator.n.changed_peek()
                    || emulator.z.changed_peek()
                    || emulator.p.changed_peek()
                {
                    egui::Color32::LIGHT_GREEN // Highlight if changed this cycle
                } else {
                    ui.visuals().text_color()
                }
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
            let mar_color = if matches!(emulator.cpu_state, CpuState::Fetch)
                || matches!(emulator.cpu_state, CpuState::EvaluateAddress(_))
                || matches!(emulator.cpu_state, CpuState::FetchOperands(_))
                || matches!(emulator.cpu_state, CpuState::StoreResult(_))
            {
                if emulator.mar.changed_peek() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    ui.visuals().text_color()
                }
            } else {
                ui.visuals().text_color()
            };
            let mdr_color = if matches!(emulator.cpu_state, CpuState::Fetch)
                || matches!(emulator.cpu_state, CpuState::FetchOperands(_))
                || matches!(emulator.cpu_state, CpuState::StoreResult(_))
            {
                if emulator.mdr.changed_peek() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    ui.visuals().text_color()
                }
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(mar_text).color(mar_color));
            ui.label(RichText::new(mdr_text).color(mdr_color));
        });

        ui.horizontal(|ui| {
            ui.label("Control:");
            let ir_text = format!("IR={:#06x}", emulator.ir.get());
            let pc_text = format!("PC={:#06x}", emulator.pc.get());
            let ir_color = if matches!(emulator.cpu_state, CpuState::Fetch) {
                if emulator.ir.changed_peek() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    egui::Color32::YELLOW
                } // Highlight if loaded, yellow if about to load
            } else if matches!(emulator.cpu_state, CpuState::Decode) {
                egui::Color32::LIGHT_GREEN // Being used in Decode
            } else {
                ui.visuals().text_color()
            };
            let pc_color = if matches!(emulator.cpu_state, CpuState::Fetch)
                || matches!(emulator.cpu_state, CpuState::EvaluateAddress(_))
                || matches!(emulator.cpu_state, CpuState::ExecuteOperation(_))
            {
                if emulator.pc.changed_peek() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    ui.visuals().text_color()
                } // PC used in Fetch, EvalAddr (for offsets), Execute (for jumps)
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(ir_text).color(ir_color));
            ui.label(RichText::new(pc_text).color(pc_color));
        });
    }
}
