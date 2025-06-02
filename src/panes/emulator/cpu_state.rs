use crate::app::EMULATOR;
use crate::emulator::ops::jsr::JsrMode;
use crate::emulator::ops::{
    AddOp, AndOp, BrOp, JmpOp, JsrOp, LdOp, LdiOp, LdrOp, LeaOp, NotOp, OpCode, RtiOp, StOp, StiOp,
    StrOp, TrapOp,
};
use crate::emulator::{BitAddressable, CpuState, Emulator, EmulatorCell, PSR_ADDR};
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use crate::theme::{ThemeSettings, CURRENT_THEME_SETTINGS};
use egui::RichText;
use serde::{Deserialize, Serialize};

use super::{editor::COMPILATION_ARTIFACTS, EmulatorPane};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CpuStatePane {}

impl PaneDisplay for CpuStatePane {
    fn render(&mut self, ui: &mut egui::Ui) {
        let theme = CURRENT_THEME_SETTINGS.lock().unwrap();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut emulator = EMULATOR.lock().unwrap();

            ui.collapsing("Flags (PSR)", |ui| {
                let (n, z, p) = emulator.get_nzp();
                let privilege_mode = emulator.priv_level();
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(n, RichText::new("N").monospace())
                        .on_hover_text("Negative Flag")
                        .clicked()
                    {
                        emulator.set_n();
                    }

                    if ui
                        .selectable_label(z, RichText::new("Z").monospace())
                        .on_hover_text("Zero Flag")
                        .clicked()
                    {
                        emulator.set_z();
                    }

                    if ui
                        .selectable_label(p, RichText::new("P").monospace())
                        .on_hover_text("Positive Flag")
                        .clicked()
                    {
                        emulator.set_p();
                    }
                    ui.separator();
                    ui.label(
                        RichText::new(match privilege_mode {
                            crate::emulator::PrivilegeLevel::Supervisor => "Supervisor Mode",
                            crate::emulator::PrivilegeLevel::User => "User Mode",
                        })
                        .monospace(),
                    );
                });
                ui.label(
                    RichText::new(format!("PSR: {:#06x}", emulator.memory[PSR_ADDR].get()))
                        .monospace()
                        .color(theme.register_value_color),
                );
            });

            ui.collapsing("Processor Cycle", |ui| {
                self.render_cycle_view(ui, &mut emulator, &theme);
            });
        });
    }

    fn title(&self) -> String {
        "CPU State".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "CPU State".to_string(),
            Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Cpu(
                CpuStatePane::default(),
            )))),
        )
    }
}

// Helper functions for RichText formatting
fn mono(text: impl Into<String>, color: egui::Color32) -> RichText {
    RichText::new(text).monospace().color(color)
}
fn reg_name_mono(text: impl Into<String>, theme: &ThemeSettings) -> RichText {
    mono(text, theme.register_name_color)
}
fn reg_val_mono(val: u16, theme: &ThemeSettings) -> RichText {
    mono(format!("{:#06x}", val), theme.register_value_color)
}
fn _reg_val_dec_mono(val: i16, theme: &ThemeSettings) -> RichText {
    mono(format!("{}", val), theme.register_value_color)
}
fn op_mono(text: impl Into<String>, theme: &ThemeSettings) -> RichText {
    mono(text, theme.secondary_text_color)
}
fn mem_addr_mono(text: impl Into<String>, theme: &ThemeSettings) -> RichText {
    mono(text, theme.memory_address_color)
}
fn desc_color(text: impl Into<String>, theme: &ThemeSettings) -> RichText {
    RichText::new(text).color(theme.cpu_state_description_color)
}

impl CpuStatePane {
    fn render_cycle_view(
        &mut self,
        ui: &mut egui::Ui,
        emulator: &mut Emulator,
        theme: &ThemeSettings,
    ) {
        let cycle_names = [
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

        let pc_for_source_lookup = if matches!(emulator.cpu_state, CpuState::Fetch) {
            emulator.pc.get()
        } else {
            emulator.pc.get().wrapping_sub(1)
        };

        let source_line_text = artifacts
            .line_to_address
            .iter()
            .find_map(|(line_num, &addr)| {
                if addr == pc_for_source_lookup as usize {
                    artifacts.last_compiled_source.lines().nth(*line_num)
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("Instruction at {:#06x}", pc_for_source_lookup));

        // Display cycle list
        ui.label(RichText::new("CPU Pipeline Stages:").strong());
        for (i, cycle_name_str) in cycle_names.iter().enumerate() {
            if i == current_cycle_index {
                ui.label(
                    RichText::new(format!("-> {}", cycle_name_str))
                        .strong()
                        .color(theme.cpu_state_active_color),
                );
            } else {
                ui.label(
                    RichText::new(format!("   {}", cycle_name_str))
                        .color(theme.secondary_text_color),
                );
            }
        }
        ui.separator();

        // Detailed description for the current cycle
        ui.label(
            RichText::new(format!(
                "Current Stage: {} for `{}` (IR: {:#06x})",
                cycle_names[current_cycle_index].to_uppercase(),
                if matches!(emulator.cpu_state, CpuState::Fetch) {
                    format!("instruction at PC={:#06x}", emulator.pc.get())
                } else {
                    source_line_text.clone()
                },
                if matches!(emulator.cpu_state, CpuState::Fetch) {
                    0u16 // IR is not yet loaded for the current PC
                } else {
                    emulator.ir.get()
                }
            ))
            .strong()
            .color(theme.primary_text_color),
        );
        ui.add_space(theme.item_spacing.y);

        ui.indent("cycle_details_indent", |ui| {
            match &emulator.cpu_state {
                CpuState::Fetch => {
                    let pc_val = emulator.pc.get();
                    ui.label(desc_color("Fetch instruction from memory and increment PC.", theme));
                    ui.horizontal_wrapped(|ui| {
                        ui.label(reg_name_mono("PC", theme));
                        ui.label(mono(format!(" ({:#06x}) ", pc_val), theme.register_value_color));
                        ui.label(op_mono("-> ", theme));
                        ui.label(reg_name_mono("MAR", theme));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(mem_addr_mono(format!("Mem[MAR] (Mem[{:#06x}]) ", pc_val), theme));
                        ui.label(op_mono("-> ", theme));
                        ui.label(reg_name_mono("MDR", theme));
                        ui.label(mono(format!(" (loads {:#06x})", emulator.memory[pc_val as usize].get()), theme.register_value_color));
                    });
                     ui.horizontal_wrapped(|ui| {
                        ui.label(reg_name_mono("MDR", theme));
                        ui.label(op_mono(" -> ", theme));
                        ui.label(reg_name_mono("IR", theme));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(reg_name_mono("PC", theme));
                        ui.label(mono(format!(" ({:#06x}) ", pc_val), theme.register_value_color));
                        ui.label(op_mono("+ 1 -> ", theme));
                        ui.label(reg_name_mono("PC", theme));
                        ui.label(mono(format!(" (becomes {:#06x})", pc_val.wrapping_add(1)), theme.register_value_color));
                    });
                }
                CpuState::Decode => {
                    let ir_val = emulator.ir;
                    let opcode_num = ir_val.range(15..12);
                    let mnemonic = OpCode::from_instruction(emulator.ir)
                                       .map(|op| format!("{}", op))
                                       .unwrap_or_else(|| "INVALID".to_string());
                    ui.label(desc_color(format!("Decode instruction in IR ({:#06x}).", ir_val.get()), theme));
                    ui.label(mono(format!("Opcode: {:#X} ({})", opcode_num.get(), mnemonic), theme.primary_text_color));

                    if let Some(decoded_op_for_display) = OpCode::from_instruction(ir_val) {
                        match decoded_op_for_display {
                            OpCode::Add(add_op_variant) => match add_op_variant {
                                AddOp::Immidiate { dr, sr1, imm5 } => {
                                    ui.label(mono(format!("  DR: R{}, SR1: R{}", dr.get(), sr1.get()), theme.secondary_text_color));
                                    ui.label(mono(format!("  imm5: {:#x} ({})", imm5.get(), imm5.sext(4).get() as i16), theme.secondary_text_color));
                                }
                                AddOp::Register { dr, sr1, sr2 } => {
                                    ui.label(mono(format!("  DR: R{}, SR1: R{}", dr.get(), sr1.get()), theme.secondary_text_color));
                                    ui.label(mono(format!("  SR2: R{}", sr2.get()), theme.secondary_text_color));
                                }
                                _ => {}
                            },
                            OpCode::And(and_op_variant) => match and_op_variant {
                                AndOp::Immediate { dr, sr1, imm5 } => {
                                    ui.label(mono(format!("  DR: R{}, SR1: R{}", dr.get(), sr1.get()), theme.secondary_text_color));
                                    ui.label(mono(format!("  imm5: {:#x} ({})", imm5.get(), imm5.sext(4).get() as i16), theme.secondary_text_color));
                                }
                                AndOp::Register { dr, sr1, sr2 } => {
                                    ui.label(mono(format!("  DR: R{}, SR1: R{}", dr.get(), sr1.get()), theme.secondary_text_color));
                                    ui.label(mono(format!("  SR2: R{}", sr2.get()), theme.secondary_text_color));
                                }
                                _ => {}
                            },
                            OpCode::Not(not_op_variant) => match not_op_variant {
                                NotOp::Decoded { dr, sr } => {
                                    ui.label(mono(format!("  DR: R{}, SR: R{}", dr.get(), sr.get()), theme.secondary_text_color));
                                }
                                _ => {}
                            },
                            OpCode::Ld(LdOp { dr, pc_offset, .. }) => {
                                ui.label(mono(format!("  DR: R{}, PCOffset9: {:#x} ({})", dr.get(), pc_offset.get(), pc_offset.sext(8).get() as i16), theme.secondary_text_color));
                            }
                            _ => {}
                        }
                    }
                }
                CpuState::EvaluateAddress(op) => {
                    ui.label(desc_color("Calculate effective memory address or target address.", theme));
                    let pc_curr = emulator.pc.get().wrapping_sub(1); // PC of current instruction
                    match op {
                        OpCode::Ld(LdOp { pc_offset, .. }) | OpCode::St(StOp { pc_offset, .. }) | OpCode::Lea(LeaOp { pc_offset, .. }) | OpCode::Ldi(LdiOp { pc_offset, .. }) | OpCode::Sti(StiOp { pc_offset, .. }) => {
                            let offset = pc_offset.sext(8).get() as i16;
                            let eff_addr = pc_curr.wrapping_add(offset as u16);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(reg_name_mono("PC", theme));
                                ui.label(mono(format!(" ({:#06x}) ", pc_curr), theme.register_value_color));
                                ui.label(op_mono("+ PCOffset9 ", theme));
                                ui.label(mono(format!("({:#06x}, dec: {}) ", offset, offset), theme.register_value_color));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(op_mono("= Effective Address ", theme));
                                ui.label(mono(format!("({:#06x}) ", eff_addr), theme.register_value_color));
                                ui.label(op_mono("-> ", theme));
                                ui.label(reg_name_mono("MAR", theme));
                            });
                        }
                        OpCode::Ldr(LdrOp { base_r, offset6, .. }) | OpCode::Str(StrOp { base_r, offset6, .. }) => {
                            let base_val = emulator.r[base_r.get() as usize].get();
                            let offset = offset6.sext(5).get() as i16;
                            let eff_addr = base_val.wrapping_add(offset as u16);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(reg_name_mono(format!("BaseR (R{})", base_r.get()), theme));
                                ui.label(mono(format!(" ({:#06x}) ", base_val), theme.register_value_color));
                                ui.label(op_mono("+ Offset6 ", theme));
                                ui.label(mono(format!("({:#06x}, dec: {}) ", offset, offset), theme.register_value_color));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(op_mono("= Effective Address ", theme));
                                ui.label(mono(format!("({:#06x}) ", eff_addr), theme.register_value_color));
                                ui.label(op_mono("-> ", theme));
                                ui.label(reg_name_mono("MAR", theme));
                            });
                        }
                        OpCode::Jsr(jsr_op) => match &jsr_op.mode {
                            JsrMode::Relative { pc_offset } => {
                                let offset = pc_offset.sext(10).get() as i16;
                                let target_addr = pc_curr.wrapping_add(offset as u16);
                                 ui.label(mono(format!("Target for JSR: PC + PCOffset11 = {:#06x}", target_addr), theme.secondary_text_color));
                            }
                            _ => { ui.label(desc_color("JSRR: Target address is from BaseR, not calculated here.", theme));}
                        },
                        OpCode::Br(BrOp { n_bit, z_bit, p_bit, pc_offset, .. }) => {
                            let (psr_n, psr_z, psr_p) = emulator.get_nzp();
                            let cond = (n_bit.get() != 0 && psr_n) || (z_bit.get() != 0 && psr_z) || (p_bit.get() != 0 && psr_p);
                            if cond {
                                let offset = pc_offset.sext(8).get() as i16;
                                let target_addr = pc_curr.wrapping_add(offset as u16);
                                ui.label(mono(format!("Branch taken. Target: PC + PCOffset9 = {:#06x}", target_addr), theme.secondary_text_color));
                            } else {
                                ui.label(mono("Branch not taken.", theme.secondary_text_color));
                            }
                        }
                        _ => { ui.label(desc_color(format!("No specific address evaluation for {}.", op), theme)); }
                    }
                }
                CpuState::FetchOperands(op) => {
                    ui.label(desc_color("Fetch operands from registers or memory.", theme));
                     match op {
                        OpCode::Add(add_op) => match add_op {
                            AddOp::Immidiate { sr1, imm5, .. } => {
                                ui.label(mono(format!("  Read SR1 (R{}): {:#06x}", sr1.get(), emulator.r[sr1.get() as usize].get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Use imm5: {:#x} ({})", imm5.get(), imm5.sext(4).get() as i16), theme.secondary_text_color));
                            }
                            AddOp::Register { sr1, sr2, .. } => {
                                ui.label(mono(format!("  Read SR1 (R{}): {:#06x}", sr1.get(), emulator.r[sr1.get() as usize].get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Read SR2 (R{}): {:#06x}", sr2.get(), emulator.r[sr2.get() as usize].get()), theme.secondary_text_color));
                            }
                            _ => {ui.label(desc_color("ADD not in expected state for fetch display.", theme));}
                        },
                        OpCode::And(and_op) => match and_op {
                            AndOp::Immediate { sr1, imm5, .. } => {
                                ui.label(mono(format!("  Read SR1 (R{}): {:#06x}", sr1.get(), emulator.r[sr1.get() as usize].get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Use imm5: {:#x} ({})", imm5.get(), imm5.sext(4).get() as i16), theme.secondary_text_color));
                            }
                            AndOp::Register { sr1, sr2, .. } => {
                                ui.label(mono(format!("  Read SR1 (R{}): {:#06x}", sr1.get(), emulator.r[sr1.get() as usize].get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Read SR2 (R{}): {:#06x}", sr2.get(), emulator.r[sr2.get() as usize].get()), theme.secondary_text_color));
                            }
                             _ => {ui.label(desc_color("AND not in expected state for fetch display.", theme));}
                        },
                        OpCode::Not(not_op) => match not_op {
                             NotOp::Decoded { sr, .. } => {
                                ui.label(mono(format!("  Read SR (R{}): {:#06x}", sr.get(), emulator.r[sr.get() as usize].get()), theme.secondary_text_color));
                             }
                             _ => {ui.label(desc_color("NOT not in Decoded state for fetch display.", theme));}
                        },
                        OpCode::Ld(_) | OpCode::Ldi(_) | OpCode::Ldr(_) => {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(mem_addr_mono(format!("Mem[MAR] (Mem[{:#06x}]) ", emulator.mar.get()), theme));
                                ui.label(op_mono("-> ", theme));
                                ui.label(reg_name_mono("MDR", theme));
                                ui.label(mono(format!(" (loads {:#06x})", emulator.memory[emulator.mar.get() as usize].get()), theme.register_value_color));
                            });
                        }
                        OpCode::St(StOp{sr, ..}) | OpCode::Sti(StiOp{sr, ..}) | OpCode::Str(StrOp{sr, ..}) => {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(reg_name_mono(format!("SR (R{})", sr.get()), theme));
                                ui.label(mono(format!(" ({:#06x}) ", emulator.r[sr.get() as usize].get()), theme.register_value_color));
                                ui.label(op_mono("-> ", theme));
                                ui.label(reg_name_mono("MDR", theme));
                            });
                        }
                        _ => { ui.label(desc_color(format!("No specific operand fetch for {}.", op), theme)); }
                    }
                }
                CpuState::ExecuteOperation(op) => {
                    ui.label(desc_color("Perform the operation using ALU, update PC for jumps/branches, set condition codes.", theme));
                    let (n,z,p) = emulator.get_nzp(); // Get flags *before* potential modification by current op
                    match op {
                        OpCode::Add(add_op) => match add_op {
                            AddOp::Ready { op1, op2, dr, .. } => {
                                let val1 = op1.get();
                                let val2 = op2.get();
                                let result = val1.wrapping_add(val2);
                                ui.label(mono(format!("  Operand1[{:#06x}] + Operand2[{:#06x}] = ALU_OUT[{:#06x}] (to R{})", val1, val2, result, dr.get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Flags (N,Z,P will be set based on {:#06x})", result), theme.secondary_text_color));
                            }
                            _ => {  ui.label(desc_color(format!("ADD not in Ready state for execute display."), theme)); }
                        },
                        OpCode::And(and_op) => match and_op {
                            AndOp::Ready { op1, op2, dr, .. } => {
                                let val1 = op1.get();
                                let val2 = op2.get();
                                let result = val1 & val2;
                                ui.label(mono(format!("  Operand1[{:#06x}] & Operand2[{:#06x}] = ALU_OUT[{:#06x}] (to R{})", val1, val2, result, dr.get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Flags (N,Z,P will be set based on {:#06x})", result), theme.secondary_text_color));
                            }
                             _ => {  ui.label(desc_color(format!("AND not in Ready state for execute display."), theme)); }
                        },
                        OpCode::Not(not_op) => match not_op {
                            NotOp::Ready { op: op1, dr, .. } => {
                                let val1 = op1.get();
                                let result = !val1;
                                ui.label(mono(format!("  NOT Operand1[{:#06x}] = ALU_OUT[{:#06x}] (to R{})", val1, result, dr.get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Flags (N,Z,P will be set based on {:#06x})", result), theme.secondary_text_color));
                            }
                            _ => {  ui.label(desc_color(format!("NOT not in Ready state for execute display."), theme)); }
                        },
                        OpCode::Jmp(JmpOp{base_r, ..}) => {
                            let target_addr = emulator.r[base_r.get() as usize].get();
                            ui.label(mono(format!("  PC := R{}[{:#06x}]", base_r.get(), target_addr), theme.secondary_text_color));
                        }
                        OpCode::Jsr(jsr_op) => {
                            ui.label(mono(format!("  R7 := PC ({:#06x})", emulator.pc.get()), theme.secondary_text_color)); // PC is already next instruction
                            match &jsr_op.mode {
                                crate::emulator::ops::jsr::JsrMode::Register { base_r } => {
                                     ui.label(mono(format!("  PC := R{}[{:#06x}]", base_r.get(), emulator.r[base_r.get() as usize].get()), theme.secondary_text_color));
                                }
                                crate::emulator::ops::jsr::JsrMode::Relative { .. } => {
                                     ui.label(mono(format!("  PC := Target Address (calculated in EvalAddr)"), theme.secondary_text_color));
                                }
                            }
                        }
                        OpCode::Br(BrOp { n_bit, z_bit, p_bit, pc_offset, .. }) => {
                            let cond = (n_bit.get() != 0 && n) || (z_bit.get() != 0 && z) || (p_bit.get() != 0 && p);
                            if cond {
                                let offset = pc_offset.sext(8).get() as i16;
                                let target_addr = emulator.pc.get().wrapping_sub(1).wrapping_add(offset as u16);
                                ui.label(mono(format!("  Branch Taken. PC := Target ({:#06x})", target_addr), theme.secondary_text_color));
                            } else {
                                ui.label(mono(format!("  Branch Not Taken. PC remains {:#06x}", emulator.pc.get()), theme.secondary_text_color));
                            }
                        }
                        OpCode::Trap(TrapOp{trap_vector, ..}) => {
                            ui.label(mono(format!("  R7 := PC ({:#06x})", emulator.pc.get()), theme.secondary_text_color));
                            ui.label(mono(format!("  PC := Mem[zext(TRAPVEC8={:#04x})]", trap_vector.get()), theme.secondary_text_color));
                        }
                        OpCode::Rti(_) => {
                             ui.label(mono("  If in supervisor mode: PC := Mem[R6]; R6 := R6+1; PSR := Mem[R6]; R6 := R6+1. Else privilege violation.", theme.secondary_text_color));
                        }
                        _ => { ui.label(desc_color(format!("No specific execution detail for {}.", op), theme)); }
                    }
                }
                CpuState::StoreResult(op) => {
                    ui.label(desc_color("Write result to register or memory, update condition codes.", theme));
                     match op {
                        OpCode::Add(add_op) => match add_op {
                            AddOp::Ready{ dr, ..} | AddOp::Immidiate { dr, .. } | AddOp::Register { dr, .. } => { // Show DR for all variants if applicable
                                ui.label(mono(format!("  ALU_OUT -> R{}", dr.get()), theme.secondary_text_color));
                                ui.label(mono("  Update PSR with new N,Z,P flags.", theme.secondary_text_color));
                            }
                        },
                        OpCode::And(and_op) => match and_op {
                            AndOp::Ready{ dr, ..} | AndOp::Immediate { dr, .. } | AndOp::Register { dr, .. } => {
                                ui.label(mono(format!("  ALU_OUT -> R{}", dr.get()), theme.secondary_text_color));
                                ui.label(mono("  Update PSR with new N,Z,P flags.", theme.secondary_text_color));
                            }
                        },
                        OpCode::Not(not_op) => match not_op {
                            NotOp::Ready{ dr, ..} | NotOp::Decoded { dr, .. } => {
                                ui.label(mono(format!("  ALU_OUT -> R{}", dr.get()), theme.secondary_text_color));
                                ui.label(mono("  Update PSR with new N,Z,P flags.", theme.secondary_text_color));
                            }
                        },
                        OpCode::Lea(LeaOp{dr, ..}) => {
                            ui.label(mono(format!("  EffectiveAddress -> R{}", dr.get()), theme.secondary_text_color));
                             ui.label(mono("  Update PSR with new N,Z,P flags.", theme.secondary_text_color));
                        }
                        OpCode::Ld(LdOp{dr, ..}) | OpCode::Ldi(LdiOp{dr, ..}) | OpCode::Ldr(LdrOp{dr, ..}) => {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(reg_name_mono("MDR", theme));
                                ui.label(mono(format!(" ({:#06x}) ", emulator.mdr.get()), theme.register_value_color));
                                ui.label(op_mono("-> ", theme));
                                ui.label(reg_name_mono(format!("R{}", dr.get()), theme));
                            });
                            ui.label(mono(format!("  Update PSR with N,Z,P flags based on R{}.", dr.get()), theme.secondary_text_color));
                        }
                        OpCode::St(_) | OpCode::Sti(_) | OpCode::Str(_) => {
                             ui.horizontal_wrapped(|ui| {
                                ui.label(reg_name_mono("MDR", theme));
                                ui.label(mono(format!(" ({:#06x}) ", emulator.mdr.get()), theme.register_value_color));
                                ui.label(op_mono("-> ", theme));
                                ui.label(mem_addr_mono(format!("Mem[MAR] (Mem[{:#06x}])", emulator.mar.get()), theme));
                            });
                        }
                        _ => { ui.label(desc_color(format!("No specific store result detail for {}.", op), theme)); }
                    }
                }
            }
        });
        ui.separator();

        ui.label(
            RichText::new("Key Registers (highlighted if changed in last micro-op):").strong(),
        );

        let active_color = theme.cpu_state_active_register_highlight;
        let default_val_color = theme.register_value_color;
        let default_name_color = theme.register_name_color;

        let reg_ui = |ui: &mut egui::Ui, name: &str, reg: &EmulatorCell, is_psr: bool| {
            let val = reg.get();
            let changed = if is_psr {
                emulator.memory[PSR_ADDR].changed_peek()
            } else {
                reg.changed_peek()
            };
            ui.label(RichText::new(name).monospace().color(if changed {
                active_color
            } else {
                default_name_color
            }));
            ui.label(
                RichText::new(format!("{:#06x}", val))
                    .monospace()
                    .color(if changed {
                        active_color
                    } else {
                        default_val_color
                    }),
            );
        };

        ui.horizontal_wrapped(|ui| {
            reg_ui(ui, "PC ", &emulator.pc, false);
            ui.add_space(theme.item_spacing.x * 2.0);
            reg_ui(ui, "IR ", &emulator.ir, false);
        });
        ui.horizontal_wrapped(|ui| {
            reg_ui(ui, "MAR", &emulator.mar, false);
            ui.add_space(theme.item_spacing.x * 2.0);
            reg_ui(ui, "MDR", &emulator.mdr, false);
        });
        ui.horizontal_wrapped(|ui| {
            let psr_val = emulator.memory[PSR_ADDR].get();
            let psr_cell = EmulatorCell::new(psr_val);
            reg_ui(ui, "PSR", &psr_cell, true);
        });

        ui.add_space(theme.item_spacing.y);
        ui.label(RichText::new("General Purpose Registers (R0-R7):").strong());
        for i in 0..8 {
            if i % 4 == 0 && i > 0 {
                ui.end_row();
            }
            reg_ui(ui, &format!("R{} ", i), &emulator.r[i], false);
            if i % 4 != 3 && i != 7 {
                ui.add_space(theme.item_spacing.x * 2.0);
            }
        }
    }
}
