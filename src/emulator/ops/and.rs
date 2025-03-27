use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct AndOp;

impl Op for AndOp {
    // The AND operation doesn't need extra memory access
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // For AND we don't need to set MAR since we only access registers
        tracing::trace!("AND: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("AND_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0101 | DR | SR1 | ImmTBit | (Register || Immediate)
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let sr1_index = ir.range(8..6).get() as usize;

        tracing::trace!(
            dr = format!("0x{:X}", dr_index),
            sr1 = format!("0x{:X}", sr1_index),
            "Extracted register indices"
        );

        // Check immediate mode (bit[5])
        if ir.index(5).get() == 0x1 {
            // Immediate mode
            let imm5 = ir.range(4..0);
            // Sign extend from 5 bits
            let imm5_val = imm5.sext(4).get();

            tracing::trace!(
                immediate = format!("0x{:X}", imm5_val),
                "Using immediate mode"
            );
            let sr1_val = machine_state.r[sr1_index].get();
            let result = sr1_val & imm5_val;
            tracing::trace!(
                sr1_value = format!("0x{:X}", sr1_val),
                imm5_value = format!("0x{:X}", imm5_val),
                result = format!("0x{:X}", result),
                "R{:X} = 0x{:X} & 0x{:X} = 0x{:X}",
                dr_index,
                sr1_val,
                imm5_val,
                result
            );

            machine_state.r[dr_index].set(result);
            tracing::trace!(
                register = format!("0x{:X}", dr_index),
                value = format!("0x{:X}", result),
                "Set register value"
            );
        } else {
            // Register mode
            let sr2_index = ir.range(2..0).get() as usize;
            tracing::trace!(sr2 = format!("0x{:X}", sr2_index), "Using register mode");

            let sr1_val = machine_state.r[sr1_index].get();
            let sr2_val = machine_state.r[sr2_index].get();
            let result = sr1_val & sr2_val;
            tracing::trace!(
                sr1_value = format!("0x{:X}", sr1_val),
                sr2_value = format!("0x{:X}", sr2_val),
                result = format!("0x{:X}", result),
                "R{:X} = 0x{:X} & 0x{:X} = 0x{:X}",
                dr_index,
                sr1_val,
                sr2_val,
                result
            );

            machine_state.r[dr_index].set(result);
            tracing::trace!(
                register = format!("0x{:X}", dr_index),
                value = format!("0x{:X}", result),
                "Set register value"
            );
        }

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = format!("0x{:X}", machine_state.n.get()),
            z = format!("0x{:X}", machine_state.z.get()),
            p = format!("0x{:X}", machine_state.p.get()),
            "Updated condition flags"
        );
    }
}
