use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct TrapOp;

impl Op for TrapOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // TRAP doesn't need extra memory access preparation in this basic implementation
        tracing::trace!("TRAP: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("TRAP_execute",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1111 | 0000 | trapvect8
        let ir = machine_state.ir;
        let trap_vector = ir.range(7..0).get();
        tracing::trace!(
            trap_vector = format!("0x{:02X}", trap_vector),
            "TRAP vector"
        );

        // Save the return address in R7
        let curr_pc = machine_state.pc.get();
        machine_state.r[7].set(curr_pc);
        tracing::trace!(
            return_address = format!("0x{:04X}", curr_pc),
            "Saving return address in R7"
        );

        // Basic implementations for common trap vectors
        match trap_vector {
            0x20 => {
                // GETC: Read a character from the keyboard
                tracing::trace!("GETC - Requesting keyboard input");
                machine_state.await_input = Some(false);
            }
            0x21 => {
                // OUT: Output a character to the console
                let char_code = machine_state.r[0].get();
                let char = char_code as u8 as char;
                tracing::trace!(
                    char_code = format!("0x{:04X}", char_code),
                    char = char.to_string(),
                    "OUT - Outputting character"
                );
                machine_state.output.push(char);
            }
            0x22 => {
                // PUTS: Output a null-terminated string starting at address in R0
                let mut string_addr = machine_state.r[0].get() as usize;
                tracing::trace!(
                    start_address = format!("0x{:04X}", string_addr),
                    "PUTS - Outputting null-terminated string"
                );

                let mut output_str = String::new();
                let mut char_count = 0;

                loop {
                    let char_value = machine_state.memory[string_addr].get();
                    if char_value == 0 {
                        break; // Null terminator found
                    }
                    let c = char_value as u8 as char;
                    output_str.push(c);
                    machine_state.output.push(c);
                    string_addr += 1;
                    char_count += 1;
                }

                tracing::trace!(
                    characters = char_count,
                    string = output_str,
                    "PUTS output string"
                );
            }
            0x23 => {
                // IN: Prompt user for input and read character
                tracing::trace!("IN - Prompting for keyboard input and waiting");
                machine_state.output.push_str("\nInput a character> ");
                machine_state.await_input = Some(true);
            }
            0x25 => {
                tracing::trace!("HALT - Halting execution");
                machine_state.running = false;
            }
            _ => {
                // For other trap vectors, in a real implementation we would
                // jump to the trap routine at the specified memory location
                tracing::trace!(
                    vector = format!("0x{:02X}", trap_vector),
                    "Unrecognized trap vector"
                );
            }
        }
    }
}
