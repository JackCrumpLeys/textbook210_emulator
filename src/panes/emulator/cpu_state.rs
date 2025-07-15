use crate::app::EMULATOR;
use crate::emulator::ops::jsr::JsrMode;
use crate::emulator::ops::{
    AddOp, AndOp, BrOp, JmpOp, LdOp, LdiOp, LdrOp, LeaOp, NotOp, OpCode, StOp, StiOp, StrOp, TrapOp,
};
use crate::emulator::{BitAddressable, CpuState, Emulator, EmulatorCell, MCR_ADDR, PSR_ADDR};
use crate::panes::emulator::cpu_state;
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use crate::theme::{ThemeSettings, CURRENT_THEME_SETTINGS};
use egui::{Response, RichText};
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CpuStatePane {
    use_negative: bool,
}

impl PaneDisplay for CpuStatePane {
    fn render(&mut self, ui: &mut egui::Ui) {
        let theme = CURRENT_THEME_SETTINGS.lock().unwrap();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut emulator = EMULATOR.lock().unwrap();

            ui.collapsing("Registers & devices", |ui| {
                self.render_register_view(ui, &mut emulator);
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
// fn reg_val_mono(val: u16, theme: &ThemeSettings) -> RichText {
//     mono(format!("{val:#06x}"), theme.register_value_color)
// }
fn _reg_val_dec_mono(val: i16, theme: &ThemeSettings) -> RichText {
    mono(format!("{val}"), theme.register_value_color)
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

fn register_view(ui: &mut egui::Ui, value_cell: &mut EmulatorCell, use_negative: bool) -> Response {
    if use_negative {
        let mut value: i16 = value_cell.get() as i16;
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(i16::MIN..=i16::MAX)
                .hexadecimal(4, false, true)
        )
        .on_hover_text("Drag to change value (hold Shift for slower). Click to edit value directly. Displayed as unsigned (-32768-32767)");

        if response.changed() {
            value_cell.set(value as u16);
        }
        response
    } else {
        let mut value: u16 = value_cell.get();
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(u16::MIN..=u16::MAX)
                .hexadecimal(4, false, true)
        )
        .on_hover_text("Drag to change value (hold Shift for slower). Click to edit value directly. Displayed as unsigned (0-65535)");

        if response.changed() {
            value_cell.set(value);
        }
        response
    }
}
impl CpuStatePane {
    fn render_register_view(&mut self, ui: &mut egui::Ui, emulator: &mut Emulator) {
        ui.checkbox(&mut self.use_negative, "Show <0 as negative")
            .on_hover_text("Wether to display the 2s complement registers as being negative if bit 15 is set. EG FFFF vs -0001.");

        egui::Grid::new("my_grid")
            .striped(true)
            .min_col_width(2.)
            .show(ui, |ui| {
                for row in 0..2 {
                    for col in 0..4 {
                        let register = row * 4 + col;
                        ui.label(format!("R{register}:"));
                        register_view(ui, &mut emulator.r[register], self.use_negative);
                    }
                    ui.end_row();
                }
                ui.label("PC:").on_hover_text("This register holds the next instruction that will be fetched in the fetch phase of the CPU. MEM[PC] -> IR");
                register_view(ui, &mut emulator.pc, self.use_negative).on_hover_text("This register holds the next instruction that will be fetched in the fetch phase of the CPU. MEM[PC] -> IR");

                ui.label("MDR:").on_hover_text("This register holds the data that has been read from memory or will be written to memory.");
                register_view(ui, &mut emulator.mdr, self.use_negative).on_hover_text("This register holds the data that has been read from memory or will be written to memory.");

                ui.label("MAR:").on_hover_text("This register holds the address of the memory location that will be read from or written to.");
                register_view(ui, &mut emulator.mar, self.use_negative).on_hover_text("This register holds the address of the memory location that will be read from or written to.");

                ui.label("IR:").on_hover_text("This register holds the instruction that has been fetched from memory. This instruction is decoded and executed by the CPU.");
                register_view(ui, &mut emulator.ir, self.use_negative).on_hover_text("This register holds the instruction that has been fetched from memory. This instruction is decoded and executed by the CPU.");
                ui.end_row();

                let (n, z, p) = emulator.get_nzp();
                let privilege_mode = emulator.priv_level();
                if ui
                    .selectable_label(n, RichText::new("N").monospace())
                    .on_hover_text("Negative Flag. This is set when the result of an arithmetic or load operation is negative.")
                    .clicked()
                {
                    emulator.set_n();
                }

                if ui
                    .selectable_label(z, RichText::new("Z").monospace())
                    .on_hover_text("Zero Flag. This is set when the result of an arithmetic or load operation is zero.")
                    .clicked()
                {
                    emulator.set_z();
                }

                if ui
                    .selectable_label(p, RichText::new("P").monospace())
                    .on_hover_text("Positive Flag. This is set when the result of an arithmetic or load operation is positive.")
                    .clicked()
                {
                    emulator.set_p();
                }
                ui.label(
                    RichText::new(match privilege_mode {
                        crate::emulator::PrivilegeLevel::Supervisor => "PRIV=0",
                        crate::emulator::PrivilegeLevel::User => "PRIV=1",
                    })
                    .monospace(),
                ).on_hover_text("Privilege Mode. This indicates the current privilege level of the CPU. PRIV=0 indicates supervisor mode, PRIV=1 indicates user mode.");

                ui.label("PSR:").on_hover_text(RichText::new("mem[0xFFFC]").code()).on_hover_text("Processor Status Register. Layout: PSR[15] = 0 when in supervisor mode and 1 when user mode, PSR[2] = N, PSR[1] = Z, PSR[0] = P");
                register_view(ui, &mut emulator.memory[PSR_ADDR], self.use_negative).on_hover_text(RichText::new("mem[0xFFFC]").code()).on_hover_text("Processor Status Register. Layout: PSR[15] = 0 when in supervisor mode and 1 when user mode, PSR[2] = N, PSR[1] = Z, PSR[0] = P");

                ui.label("MCR:").on_hover_text(RichText::new("mem[0xFFFE]").code()).on_hover_text("Machine Control Register, when MCR[15] is set the machine is running, otherwise it is halted");
                register_view(ui, &mut emulator.memory[MCR_ADDR], self.use_negative).on_hover_text(RichText::new("mem[0xFFFE]").code()).on_hover_text("Machine Control Register, when MCR[15] is set the machine is running, otherwise it is halted");
                ui.end_row();

               ui.label("KBDR:").on_hover_text(RichText::new("mem[0xFE02]").code()).on_hover_text("Keyboard Data Register, contains the last typed ASCII character in bits [7:0].");
               register_view(ui, &mut emulator.memory[0xFE02], self.use_negative).on_hover_text(RichText::new("mem[0xFE02]").code()).on_hover_text("Keyboard Data Register, contains the last typed ASCII character in bits [7:0].");

               ui.label("KBSR:").on_hover_text(RichText::new("mem[0xFE00]").code()).on_hover_text("Keyboard Status Register, KBSR[15] = 1 when there is input to be read.");
               register_view(ui, &mut emulator.memory[0xFE00], self.use_negative).on_hover_text(RichText::new("mem[0xFE00]").code()).on_hover_text("Keyboard Status Register, KBSR[15] = 1 when there is input to be read.");

               ui.label("DSR:").on_hover_text(RichText::new("mem[0xFE04]").code()).on_hover_text("Display Status Register, DSR[15] = 1 when display service is ready to display a new character (always 1 in this emulator).");
               register_view(ui, &mut emulator.memory[0xFE04], self.use_negative).on_hover_text(RichText::new("mem[0xFE04]").code()).on_hover_text("Display Status Register, DSR[15] = 1 when display service is ready to display a new character (always 1 in this emulator).");

               ui.label("DDR:").on_hover_text(RichText::new("mem[0xFE06]").code()).on_hover_text("Display Data Register, when DDR[7:0] is set we write the ASCII character contained to the output.");
               register_view(ui, &mut emulator.memory[0xFE06], self.use_negative).on_hover_text(RichText::new("mem[0xFE06]").code()).on_hover_text("Display Data Register, when DDR[7:0] is set we write the ASCII character contained to the output.");
            });
    }

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
        // Display cycle list
        ui.label(RichText::new("CPU Pipeline Stages:").strong());
        for (i, cycle_name_str) in cycle_names.iter().enumerate() {
            if i == current_cycle_index {
                ui.label(
                    RichText::new(format!("-> {cycle_name_str}"))
                        .strong()
                        .color(theme.cpu_state_active_color),
                );
            } else {
                ui.label(
                    RichText::new(format!("   {cycle_name_str}")).color(theme.secondary_text_color),
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
                    format!("{:?}", emulator.cpu_state)
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
                        ui.label(mono(format!(" ({pc_val:#06x}) "), theme.register_value_color));
                        ui.label(op_mono("-> ", theme));
                        ui.label(reg_name_mono("MAR", theme));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(mem_addr_mono(format!("Mem[MAR] (Mem[{pc_val:#06x}]) "), theme));
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
                        ui.label(mono(format!(" ({pc_val:#06x}) "), theme.register_value_color));
                        ui.label(op_mono("+ 1 -> ", theme));
                        ui.label(reg_name_mono("PC", theme));
                        ui.label(mono(format!(" (becomes {:#06x})", pc_val.wrapping_add(1)), theme.register_value_color));
                    });
                }
                CpuState::Decode => {
                    let ir_val = emulator.ir;
                    #[allow(clippy::reversed_empty_ranges)]
                    let opcode_num = ir_val.range(15..12);
                    let mnemonic = OpCode::from_instruction(emulator.ir)
                                       .map(|op| format!("{op}"))
                                       .unwrap_or_else(|| "INVALID".to_string());
                    ui.label(desc_color(format!("Decode instruction in IR ({:#06x}).", ir_val.get()), theme));
                    ui.label(mono(format!("Opcode: {:#X} ({})", opcode_num.get(), mnemonic), theme.primary_text_color));


                    ui.label(mono(format!("{}", emulator.cpu_state), theme.secondary_text_color));

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
                            OpCode::Not(NotOp::Decoded { dr, sr }) => {
                                ui.label(mono(format!("  DR: R{}, SR: R{}", dr.get(), sr.get()), theme.secondary_text_color));
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
                    ui.label(mono(format!("{}", emulator.cpu_state), theme.secondary_text_color));
                    match op {
                        OpCode::Ld(LdOp { pc_offset, .. }) | OpCode::St(StOp { pc_offset, .. }) | OpCode::Lea(LeaOp { pc_offset, .. }) | OpCode::Ldi(LdiOp { pc_offset, .. }) | OpCode::Sti(StiOp { pc_offset, .. }) => {
                            let offset = pc_offset.sext(8).get() as i16;
                            let eff_addr = pc_curr.wrapping_add(offset as u16);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(reg_name_mono("PC", theme));
                                ui.label(mono(format!(" ({pc_curr:#06x}) "), theme.register_value_color));
                                ui.label(op_mono("+ PCOffset9 ", theme));
                                ui.label(mono(format!("({offset:#06x}, dec: {offset}) "), theme.register_value_color));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(op_mono("= Effective Address ", theme));
                                ui.label(mono(format!("({eff_addr:#06x}) "), theme.register_value_color));
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
                                ui.label(mono(format!(" ({base_val:#06x}) "), theme.register_value_color));
                                ui.label(op_mono("+ Offset6 ", theme));
                                ui.label(mono(format!("({offset:#06x}, dec: {offset}) "), theme.register_value_color));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(op_mono("= Effective Address ", theme));
                                ui.label(mono(format!("({eff_addr:#06x}) "), theme.register_value_color));
                                ui.label(op_mono("-> ", theme));
                                ui.label(reg_name_mono("MAR", theme));
                            });
                        }
                        OpCode::Jsr(jsr_op) => match &jsr_op.mode {
                            JsrMode::Relative { pc_offset } => {
                                let offset = pc_offset.sext(10).get() as i16;
                                let target_addr = pc_curr.wrapping_add(offset as u16);
                                 ui.label(mono(format!("Target for JSR: PC + PCOffset11 = {target_addr:#06x}"), theme.secondary_text_color));
                            }
                            _ => { ui.label(desc_color("JSRR: Target address is from BaseR, not calculated here.", theme));}
                        },
                        OpCode::Br(BrOp { n_bit, z_bit, p_bit, pc_offset, .. }) => {
                            let (psr_n, psr_z, psr_p) = emulator.get_nzp();
                            let cond = (n_bit.get() != 0 && psr_n) || (z_bit.get() != 0 && psr_z) || (p_bit.get() != 0 && psr_p);
                            if cond {
                                let offset = pc_offset.sext(8).get() as i16;
                                let target_addr = pc_curr.wrapping_add(offset as u16);
                                ui.label(mono(format!("Branch taken. Target: PC + PCOffset9 = {target_addr:#06x}"), theme.secondary_text_color));
                            } else {
                                ui.label(mono("Branch not taken.", theme.secondary_text_color));
                            }
                        }
                        _ => { ui.label(desc_color(format!("No specific address evaluation for {op}."), theme)); }
                    }
                }
                CpuState::FetchOperands(op) => {
                    ui.label(desc_color("Fetch operands from registers or memory.", theme));
                    ui.label(mono(format!("{}", emulator.cpu_state), theme.secondary_text_color));
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
                        _ => { ui.label(desc_color(format!("No specific operand fetch for {op}."), theme)); }
                    }
                }
                CpuState::ExecuteOperation(op) => {
                    ui.label(desc_color("Perform the operation using ALU, update PC for jumps/branches, set condition codes.", theme));
                    ui.label(mono(format!("{}", emulator.cpu_state), theme.secondary_text_color));
                    let (n,z,p) = emulator.get_nzp(); // Get flags *before* potential modification by current op
                    match op {
                        OpCode::Add(add_op) => match add_op {
                            AddOp::Ready { op1, op2, dr, .. } => {
                                let val1 = op1.get();
                                let val2 = op2.get();
                                let result = val1.wrapping_add(val2);
                                ui.label(mono(format!("  Operand1[{:#06x}] + Operand2[{:#06x}] = ALU_OUT[{:#06x}] (to R{})", val1, val2, result, dr.get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Flags (N,Z,P will be set based on {result:#06x})"), theme.secondary_text_color));
                            }
                            _ => {  ui.label(desc_color("ADD not in Ready state for execute display.".to_string(), theme)); }
                        },
                        OpCode::And(and_op) => match and_op {
                            AndOp::Ready { op1, op2, dr, .. } => {
                                let val1 = op1.get();
                                let val2 = op2.get();
                                let result = val1 & val2;
                                ui.label(mono(format!("  Operand1[{:#06x}] & Operand2[{:#06x}] = ALU_OUT[{:#06x}] (to R{})", val1, val2, result, dr.get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Flags (N,Z,P will be set based on {result:#06x})"), theme.secondary_text_color));
                            }
                             _ => {  ui.label(desc_color("AND not in Ready state for execute display.".to_string(), theme)); }
                        },
                        OpCode::Not(not_op) => match not_op {
                            NotOp::Ready { op: op1, dr, .. } => {
                                let val1 = op1.get();
                                let result = !val1;
                                ui.label(mono(format!("  NOT Operand1[{:#06x}] = ALU_OUT[{:#06x}] (to R{})", val1, result, dr.get()), theme.secondary_text_color));
                                ui.label(mono(format!("  Flags (N,Z,P will be set based on {result:#06x})"), theme.secondary_text_color));
                            }
                            _ => {  ui.label(desc_color("NOT not in Ready state for execute display.".to_string(), theme)); }
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
                                     ui.label(mono("  PC := Target Address (calculated in EvalAddr)".to_string(), theme.secondary_text_color));
                                }
                            }
                        }
                        OpCode::Br(BrOp { n_bit, z_bit, p_bit, pc_offset, .. }) => {
                            let cond = (n_bit.get() != 0 && n) || (z_bit.get() != 0 && z) || (p_bit.get() != 0 && p);
                            if cond {
                                let offset = pc_offset.sext(8).get() as i16;
                                let target_addr = emulator.pc.get().wrapping_sub(1).wrapping_add(offset as u16);
                                ui.label(mono(format!("  Branch Taken. PC := Target ({target_addr:#06x})"), theme.secondary_text_color));
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
                        _ => { ui.label(desc_color(format!("No specific execution detail for {op}."), theme)); }
                    }
                }
                CpuState::StoreResult(op) => {
                    ui.label(desc_color("Write result to register or memory, update condition codes.", theme));
                    ui.label(mono(format!("{}", emulator.cpu_state), theme.secondary_text_color));
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
                        _ => { ui.label(desc_color(format!("No specific store result detail for {op}."), theme)); }
                    }
                }
            }
        });
        ui.collapsing("Spooky dev full cpu state.", |ui| {
            egui_extras::syntax_highlighting::code_view_ui(
                ui,
                &egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style()),
                format!("{:#?}", emulator.cpu_state).as_str(),
                "rs",
            );
        });
    }
}
