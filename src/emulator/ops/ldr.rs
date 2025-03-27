use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct LdrOp;

impl Op for LdrOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDR_prepare_memory",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0110 | DR | BaseR | offset6
        let ir = machine_state.ir;

        // Get base register index
        let base_r_index = ir.range(8..6).get() as usize;
        let base_r_value = machine_state.r[base_r_index].get();

        // Calculate effective address: BaseR + offset6
        let offset = ir.range(5..0).sext(5).get();
        let effective_address = base_r_value.wrapping_add(offset);

        tracing::trace!(
            base_register = base_r_index,
            base_value = format!("0x{:04X}", base_r_value),
            offset = format!("0x{:04X}", offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for register-based load"
        );

        // Set MAR to the effective address
        machine_state.mar.set(effective_address);
        tracing::trace!(
            mar = format!("0x{:04X}", effective_address),
            "Setting MAR for memory access"
        );
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDR_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0110 | DR | BaseR | offset6
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let effective_address = machine_state.mar.get();

        // MDR was loaded during memory access phase
        let value = machine_state.mdr.get();
        tracing::trace!(
            address = format!("0x{:04X}", effective_address),
            value = format!("0x{:04X}", value),
            dest_register = dr_index,
            "Loading value from register-relative address into register"
        );
        machine_state.r[dr_index].set(value);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after register-based load"
        );
    }
}
