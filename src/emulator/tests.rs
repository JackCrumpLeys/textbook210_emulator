use super::*;
use ops::*;
use parse::ParseOutput;
use tracing_test::traced_test;

#[traced_test]
#[test]
fn test_add_op() {
    tracing::info_span!("test_add_op").in_scope(|| {
        tracing::info!("Starting ADD operation test");

        let mut machine_state = Emulator::new();
        machine_state.r[0].set(5);
        tracing::debug!(register = 0, value = 5, "Initialized register");
        machine_state.r[1].set(3);
        tracing::debug!(register = 1, value = 3, "Initialized register");

        // Set instruction register for ADD R0, R0, R1
        // 0001 (ADD) | 000 (DR=R0) | 000 (SR1=R0) | 0 (not immediate) | 00 | 001 (SR2=R1)
        machine_state.ir.set(0b0001_000_000_0_00_001);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let add_op = AddOp;
        tracing::debug!("Executing ADD operation");
        add_op.execute(&mut machine_state);
        tracing::debug!("ADD operation executed");

        tracing::debug!(result = machine_state.r[0].get(), "Final R0 value");
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[0].get(), 8);
        assert_eq!(machine_state.n.get(), 0);
        assert_eq!(machine_state.z.get(), 0);
        assert_eq!(machine_state.p.get(), 1);
        tracing::info!("ADD operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_and_op() {
    tracing::info_span!("test_and_op").in_scope(|| {
        tracing::info!("Starting AND operation test");

        let mut machine_state = Emulator::new();
        machine_state.r[0].set(0b1010);
        tracing::debug!(register = 0, value = 0b1010, "Initialized register");
        machine_state.r[1].set(0b1100);
        tracing::debug!(register = 1, value = 0b1100, "Initialized register");

        // Set instruction register for AND R2, R0, R1
        // 0101 (AND) | 010 (DR=R2) | 000 (SR1=R0) | 0 (not immediate) | 00 | 001 (SR2=R1)
        machine_state.ir.set(0b0101_010_000_0_00_001);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let and_op = AndOp;
        tracing::debug!("Executing AND operation");
        and_op.execute(&mut machine_state);
        tracing::debug!("AND operation executed");

        tracing::debug!(
            result = format!("0b{:b}", machine_state.r[2].get()),
            "Final R2 value"
        );
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[2].get(), 0b1000);
        assert_eq!(machine_state.n.get(), 0);
        assert_eq!(machine_state.z.get(), 0);
        assert_eq!(machine_state.p.get(), 1);
        tracing::info!("AND operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_br_op() {
    tracing::info_span!("test_br_op").in_scope(|| {
        tracing::info!("Starting BR operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.n.set(1);
        machine_state.z.set(0);
        machine_state.p.set(0);
        tracing::debug!(n = 1, z = 0, p = 0, "Set condition codes");

        // Set instruction register for BRn #5 (branch if negative)
        // 0000 (BR) | 1 (n) | 0 (z) | 0 (p) | 000000101 (offset=5)
        machine_state.ir.set(0b0000_100_000000101);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let br_op = BrOp;
        tracing::debug!("Executing BR operation");
        br_op.execute(&mut machine_state);
        tracing::debug!("BR operation executed");

        tracing::debug!(
            pc = format!("0x{:04X}", machine_state.pc.get()),
            "Final program counter"
        );
        assert_eq!(machine_state.pc.get(), 0x3005);
        tracing::info!("BR operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_jmp_op() {
    tracing::info_span!("test_jmp_op").in_scope(|| {
        tracing::info!("Starting JMP operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.r[2].set(0x4000);
        tracing::debug!(
            register = 2,
            value = format!("0x{:04X}", 0x4000),
            "Initialized register"
        );

        // Set instruction register for JMP R2
        // 1100 (JMP) | 000 | 010 (BaseR=R2) | 000000
        machine_state.ir.set(0b1100_000_010_000000);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let jmp_op = JmpOp;
        tracing::debug!("Executing JMP operation");
        jmp_op.execute(&mut machine_state);
        tracing::debug!("JMP operation executed");

        tracing::debug!(
            pc = format!("0x{:04X}", machine_state.pc.get()),
            "Final program counter"
        );
        assert_eq!(machine_state.pc.get(), 0x4000);
        tracing::info!("JMP operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_jsr_op() {
    tracing::info_span!("test_jsr_op").in_scope(|| {
        tracing::info!("Starting JSR operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        // Set instruction register for JSR #10
        // 0100 (JSR) | 1 (JSR mode) | 00000001010 (offset=10)
        machine_state.ir.set(0b0100_1_00000001010);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let jsr_op = JsrOp;
        tracing::debug!("Executing JSR operation");
        jsr_op.execute(&mut machine_state);
        tracing::debug!("JSR operation executed");

        tracing::debug!(
            pc = format!("0x{:04X}", machine_state.pc.get()),
            r7 = format!("0x{:04X}", machine_state.r[7].get()),
            "Final state after JSR"
        );

        assert_eq!(machine_state.r[7].get(), 0x3000);
        assert_eq!(machine_state.pc.get(), 0x300A);
        tracing::info!("JSR operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_ld_op() {
    tracing::info_span!("test_ld_op").in_scope(|| {
        tracing::info!("Starting LD operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.memory[0x3005].set(0x1234);
        tracing::debug!(
            address = format!("0x{:04X}", 0x3005),
            value = format!("0x{:04X}", 0x1234),
            "Set memory value"
        );

        // Set instruction register for LD R3, #5
        // 0010 (LD) | 011 (DR=R3) | 000000101 (offset=5)
        machine_state.ir.set(0b0010_011_000000101);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let ld_op = LdOp;
        tracing::debug!("Preparing memory access for LD operation");
        ld_op.prepare_memory_access(&mut machine_state);
        tracing::debug!(
            mar = format!("0x{:04X}", machine_state.mar.get()),
            "MAR set for memory access"
        );

        machine_state
            .mdr
            .set(machine_state.memory[machine_state.mar.get() as usize].get());
        tracing::debug!(
            mdr = format!("0x{:04X}", machine_state.mdr.get()),
            "MDR loaded with memory value"
        );

        tracing::debug!("Executing LD operation");
        ld_op.execute(&mut machine_state);
        tracing::debug!("LD operation executed");

        tracing::debug!(
            r3 = format!("0x{:04X}", machine_state.r[3].get()),
            "Final register value"
        );
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[3].get(), 0x1234);
        assert_eq!(machine_state.p.get(), 1);
        tracing::info!("LD operation test completed successfully");
    });
}
#[traced_test]
#[test]
fn test_ldi_op() {
    tracing::info_span!("test_ldi_op").in_scope(|| {
        tracing::info!("Starting LDI operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.memory[0x3005].set(0x4000);
        tracing::debug!(
            address = format!("0x{:04X}", 0x3005),
            value = format!("0x{:04X}", 0x4000),
            "Set pointer address in memory"
        );

        machine_state.memory[0x4000].set(0x5678);
        tracing::debug!(
            address = format!("0x{:04X}", 0x4000),
            value = format!("0x{:04X}", 0x5678),
            "Set target value in memory"
        );

        // Set instruction register for LDI R4, #5
        // 1010 (LDI) | 100 (DR=R4) | 000000101 (offset=5)
        machine_state.ir.set(0b1010_100_000000101);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let ldi_op = LdiOp;
        tracing::debug!("Preparing memory access for LDI operation");
        ldi_op.prepare_memory_access(&mut machine_state);
        tracing::debug!(
            mar = format!("0x{:04X}", machine_state.mar.get()),
            mdr = format!("0x{:04X}", machine_state.mdr.get()),
            "Memory registers for indirect addressing"
        );

        machine_state
            .mdr
            .set(machine_state.memory[machine_state.mar.get() as usize].get());
        tracing::debug!(
            address = format!("0x{:04X}", machine_state.mar.get()),
            value = format!("0x{:04X}", machine_state.mdr.get()),
            "Loaded indirect value from memory"
        );

        tracing::debug!("Executing LDI operation");
        ldi_op.execute(&mut machine_state);
        tracing::debug!("LDI operation executed");

        tracing::debug!(
            r4 = format!("0x{:04X}", machine_state.r[4].get()),
            "Final register value"
        );
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[4].get(), 0x5678);
        assert_eq!(machine_state.p.get(), 1);
        tracing::info!("LDI operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_ldr_op() {
    tracing::info_span!("test_ldr_op").in_scope(|| {
        tracing::info!("Starting LDR operation test");

        let mut machine_state = Emulator::new();
        machine_state.r[2].set(0x4000);
        tracing::debug!(
            register = 2,
            value = format!("0x{:04X}", 0x4000),
            "Initialized base register"
        );

        machine_state.memory[0x4003].set(0x9ABC);
        tracing::debug!(
            address = format!("0x{:04X}", 0x4003),
            value = format!("0x{:04X}", 0x9ABC),
            "Set memory value"
        );

        // Set instruction register for LDR R5, R2, #3
        // 0110 (LDR) | 101 (DR=R5) | 010 (BaseR=R2) | 000011 (offset=3)
        machine_state.ir.set(0b0110_101_010_000011);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            "Set instruction register"
        );

        let ldr_op = LdrOp;
        tracing::debug!("Preparing memory access for LDR operation");
        ldr_op.prepare_memory_access(&mut machine_state);
        tracing::debug!(
            base_register = 2,
            offset = 3,
            effective_address = format!("0x{:04X}", machine_state.mar.get()),
            "Calculated effective address for LDR"
        );

        machine_state
            .mdr
            .set(machine_state.memory[machine_state.mar.get() as usize].get());
        tracing::debug!(
            address = format!("0x{:04X}", machine_state.mar.get()),
            value = format!("0x{:04X}", machine_state.mdr.get()),
            "Loaded value from memory"
        );

        tracing::debug!("Executing LDR operation");
        ldr_op.execute(&mut machine_state);
        tracing::debug!("LDR operation executed");

        tracing::debug!(
            r5 = format!("0x{:04X}", machine_state.r[5].get()),
            "Final register value"
        );
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[5].get(), 0x9ABC);
        assert_eq!(machine_state.n.get(), 1); // bit 15 is set as 9 = 0b1001
        tracing::info!("LDR operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_lea_op() {
    tracing::info_span!("test_lea_op").in_scope(|| {
        tracing::info!("Starting LEA operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        // Set instruction register for LEA R6, #8
        // 1110 (LEA) | 110 (DR=R6) | 000001000 (offset=8)
        machine_state.ir.set(0b1110_110_000001000);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            dr = 6,
            offset = 8,
            "Set instruction register for LEA"
        );

        let lea_op = LeaOp;
        tracing::debug!("Executing LEA operation");
        lea_op.execute(&mut machine_state);
        tracing::debug!("LEA operation executed");

        tracing::debug!(
            r6 = format!("0x{:04X}", machine_state.r[6].get()),
            expected = format!("0x{:04X}", 0x3008),
            "Final register value"
        );
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[6].get(), 0x3008);
        assert_eq!(machine_state.p.get(), 1);
        tracing::info!("LEA operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_not_op() {
    tracing::info_span!("test_not_op").in_scope(|| {
        tracing::info!("Starting NOT operation test");

        let mut machine_state = Emulator::new();
        machine_state.r[1].set(0xAAAA);
        tracing::debug!(
            register = 1,
            value = format!("0x{:04X}", 0xAAAA),
            binary = format!("0b{:016b}", 0xAAAA),
            "Initialized register with pattern 10101010..."
        );

        // Set instruction register for NOT R2, R1
        // 1001 (NOT) | 010 (DR=R2) | 001 (SR=R1) | 111111
        machine_state.ir.set(0b1001_010_001_111111);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            dr = 2,
            sr = 1,
            "Set instruction register for NOT"
        );

        let not_op = NotOp;
        tracing::debug!("Executing NOT operation");
        not_op.execute(&mut machine_state);
        tracing::debug!("NOT operation executed");

        tracing::debug!(
            r2 = format!("0x{:04X}", machine_state.r[2].get()),
            binary = format!("0b{:016b}", machine_state.r[2].get()),
            "Final register value after NOT"
        );
        tracing::debug!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Final condition flags"
        );

        assert_eq!(machine_state.r[2].get(), 0x5555);
        assert_eq!(machine_state.p.get(), 1);
        assert_eq!(machine_state.n.get(), 0);
        assert_eq!(machine_state.z.get(), 0);
        tracing::info!("NOT operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_st_op() {
    tracing::info_span!("test_st_op").in_scope(|| {
        tracing::info!("Starting ST operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.r[3].set(0xDEAD);
        tracing::debug!(
            register = 3,
            value = format!("0x{:04X}", 0xDEAD),
            "Initialized register with value to store"
        );

        // Set instruction register for ST R3, #6
        // 0011 (ST) | 011 (SR=R3) | 000000110 (offset=6)
        machine_state.ir.set(0b0011_011_000000110);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            sr = 3,
            offset = 6,
            "Set instruction register for ST"
        );

        let st_op = StOp;
        tracing::debug!("Executing ST operation");
        st_op.execute(&mut machine_state);
        tracing::debug!("ST operation executed");

        let memory_addr = 0x3006;
        tracing::debug!(
            address = format!("0x{:04X}", memory_addr),
            value = format!("0x{:04X}", machine_state.memory[memory_addr].get()),
            "Memory value after store operation"
        );

        assert_eq!(machine_state.memory[0x3006].get(), 0xDEAD);
        tracing::info!("ST operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_sti_op() {
    tracing::info_span!("test_sti_op").in_scope(|| {
        tracing::info!("Starting STI operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.r[4].set(0xBEEF);
        tracing::debug!(
            register = 4,
            value = format!("0x{:04X}", 0xBEEF),
            "Initialized source register"
        );

        machine_state.memory[0x3007].set(0x4000);
        tracing::debug!(
            address = format!("0x{:04X}", 0x3007),
            value = format!("0x{:04X}", 0x4000),
            "Set pointer address in memory"
        );

        // Set instruction register for STI R4, #7
        // 1011 (STI) | 100 (SR=R4) | 000000111 (offset=7)
        machine_state.ir.set(0b1011_100_000000111);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            sr = 4,
            offset = 7,
            "Set instruction register for STI"
        );

        let sti_op = StiOp;
        tracing::debug!("Preparing memory access for STI operation");
        sti_op.prepare_memory_access(&mut machine_state);
        tracing::debug!(
            mar = format!("0x{:04X}", machine_state.mar.get()),
            "MAR set to pointer address"
        );

        machine_state
            .mdr
            .set(machine_state.memory[machine_state.mar.get() as usize].get());
        tracing::debug!(
            mdr = format!("0x{:04X}", machine_state.mdr.get()),
            "MDR loaded with pointer value"
        );

        tracing::debug!("Executing STI operation");
        sti_op.execute(&mut machine_state);
        tracing::debug!("STI operation completed");

        let target_address = 0x4000;
        tracing::debug!(
            address = format!("0x{:04X}", target_address),
            value = format!("0x{:04X}", machine_state.memory[target_address].get()),
            "Final memory value after indirect store"
        );

        assert_eq!(machine_state.memory[0x4000].get(), 0xBEEF);
        tracing::info!("STI operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_str_op() {
    tracing::info_span!("test_str_op").in_scope(|| {
        tracing::info!("Starting STR operation test");

        let mut machine_state = Emulator::new();
        machine_state.r[5].set(0xCAFE);
        tracing::debug!(
            register = 5,
            value = format!("0x{:04X}", 0xCAFE),
            "Initialized source register"
        );

        machine_state.r[1].set(0x5000);
        tracing::debug!(
            register = 1,
            value = format!("0x{:04X}", 0x5000),
            "Initialized base register"
        );

        // Set instruction register for STR R5, R1, #4
        // 0111 (STR) | 101 (SR=R5) | 001 (BaseR=R1) | 000100 (offset=4)
        machine_state.ir.set(0b0111_101_001_000100);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            sr = 5,
            base_register = 1,
            offset = 4,
            "Set instruction register for STR"
        );

        let str_op = StrOp;
        tracing::debug!("Executing STR operation");
        str_op.execute(&mut machine_state);
        tracing::debug!("STR operation completed");

        let target_address = 0x5004;
        tracing::debug!(
            address = format!("0x{:04X}", target_address),
            value = format!("0x{:04X}", machine_state.memory[target_address].get()),
            expected = format!("0x{:04X}", 0xCAFE),
            "Memory value after register-relative store"
        );

        assert_eq!(machine_state.memory[0x5004].get(), 0xCAFE);
        tracing::info!("STR operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_trap_op() {
    tracing::info_span!("test_trap_op").in_scope(|| {
        tracing::info!("Starting TRAP operation test");

        let mut machine_state = Emulator::new();
        machine_state.pc.set(0x3000);
        tracing::debug!(
            pc = format!("0x{:04X}", 0x3000),
            "Initialized program counter"
        );

        machine_state.r[0].set(0x41); // ASCII 'A'
        tracing::debug!(
            register = 0,
            value = format!("0x{:04X}", 0x41),
            "Initialized R0 with ASCII character 'A'"
        );

        // Set instruction register for TRAP x21 (OUT)
        // 1111 (TRAP) | 0000 | 00100001 (trapvect=x21)
        machine_state.ir.set(0b1111_0000_00100001);
        tracing::debug!(
            ir = format!("0b{:016b}", machine_state.ir.get()),
            trap_vector = format!("0x{:02X}", 0x21),
            "Set instruction register for TRAP OUT"
        );

        let trap_op = TrapOp;
        tracing::debug!("Executing TRAP operation");
        trap_op.execute(&mut machine_state);
        tracing::debug!("TRAP operation completed");

        tracing::debug!(
            return_address = format!("0x{:04X}", machine_state.r[7].get()),
            output_size = machine_state.output.len(),
            output_value = format!(
                "0x{:04X}",
                machine_state.output.chars().next().unwrap() as u32
            ),
            "State after TRAP operation"
        );

        assert_eq!(machine_state.r[7].get(), 0x3000);
        assert_eq!(machine_state.output.len(), 1);
        assert_eq!(machine_state.output.chars().next(), Some('A'));
        tracing::info!("TRAP operation test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_full_program_execution() {
    tracing::info_span!("test_full_program_execution").in_scope(|| {
        tracing::info!("Starting comprehensive program execution test with all instructions");

        let program = r#"
            .ORIG x3000

            ; Initialize registers
            AND R0, R0, #0      ; Clear R0
            AND R1, R1, #0      ; Clear R1
            AND R2, R2, #0      ; Clear R2
            AND R3, R3, #0      ; Clear R3
            AND R4, R4, #0      ; Clear R4
            AND R5, R5, #0      ; Clear R5
            AND R6, R6, #0      ; Clear R6

            ; Load indirect and set up base register
            LDI R1, POINTER     ; Load R1 indirectly from pointer
            ADD R2, R1, #10     ; Add an immediate to R1

            ; Test branching
            ADD R3, R3, #1      ; Set R3 to 1
            BRp SKIP_SECTION    ; Branch if positive

            ; This section should be skipped
            ADD R3, R3, #15     ; Would set R3 to 16 if executed

            SKIP_SECTION:
            ADD R3, R3, #2      ; R3 now equals 3

            ; Test bitwise operations
            NOT R4, R3          ; Complement R3 into R4
            AND R5, R4, R1      ; Bitwise AND R4 and R1

            ; Load effective address
            LEA R6, SUBROUTINE  ; Load address of subroutine into R6

            ; Test register indirect jump
            JSRR R6             ; Jump to subroutine at address in R6

            ; After return, do a relative jump
            JSR SECOND_SUB      ; Jump to second subroutine using PC-relative

            ; After return from both subroutines, test load/store
            LD R1, DATA_VAL      ; Load address of DATA_VAL
            ADD R0, R0, R1       ; Add DATA_VAL address to R0
            ST R0, DATA_VAL      ; Store directly
            LD R0, DATA_VAL      ; Load value directly
            STR R0, R2, #0       ; Store using register-based addressing

            ; Load register-based
            LDR R3, R2, #0      ; Load the value we just stored

            ; Test NOT and store
            NOT R3, R3          ; Complement the value
            ST R3, RESULT       ; Store directly

            ; Test indirect store
            LEA R1, RESULT_PTR  ; Load address of result pointer
            STI R0, RESULT_PTR  ; Store indirectly


            ; Jump unconditionally past the data
            BRNZP END_PROG

            ; Data section should be skipped
            DATA_VAL: .FILL x0041    ; ASCII 'A' (65)
            RESULT:   .FILL x0000    ; To store direct result
            POINTER:  .FILL PTR_VAL  ; Pointer to a value
            PTR_VAL:  .FILL x00BE    ; Value 190 to load indirectly
            RESULT_PTR: .FILL INDIRECT_RESULT ; Pointer to indirect result
            INDIRECT_RESULT: .FILL x0000 ; To store indirect result

            ; First subroutine
            SUBROUTINE:
                ADD R0, R0, #5      ; Add 5 to R0
                RET                 ; Return using R7

            ; Second subroutine
            SECOND_SUB:
                ADD R0, R0, #3      ; Add 3 to R0
                AND R5, R5, #0      ; Clear R5
                ADD R5, R5, #1      ; Set R5 to 1
                BRnzp RETURN_SUB    ; Branch always to return
                ADD R5, R5, #10     ; Shouldn't be executed

            RETURN_SUB:
                RET                 ; Return using R7

            END_PROG:
                OUT                 ; Output the character in R0
                HALT                ; Stop execution


            REN: .fill #-10

            .END
            "#;

        tracing::debug!(
            program = program,
            "Complex assembled program to test all instructions"
        );

        // Parse the program
        tracing::debug!("Parsing program");
        let parse_result = Emulator::parse_program(program);

        // Check if parsing was successful
        assert!(parse_result.is_ok(), "Program parsing should succeed");

        let ParseOutput {
            labels,
            orig_address,
            machine_code,
            ..
        } = parse_result.unwrap();

        // Create an emulator and load the program
        let mut machine_state = Emulator::new();
        tracing::debug!("Loading program into emulator");
        machine_state.flash_memory(machine_code, orig_address);

        // Verify the program was loaded correctly
        assert_eq!(
            machine_state.pc.get(),
            orig_address,
            "PC should be set to origin address"
        );

        // Execute the program with a maximum number of steps
        tracing::debug!("Beginning program execution");
        let max_steps = 100; // Prevent infinite loops
        machine_state.running = true;
        let result = machine_state.run(Some(max_steps));

        // Verify execution completed successfully
        assert!(result.is_ok(), "Program execution should succeed");

        // Verify the machine halted
        assert!(!machine_state.running, "Machine should have halted");

        // Verify the results in memory
        let direct_result_address = *labels.get("RESULT").unwrap();
        let expected_direct_result = 0xFFB6;

        // 2. Register-based store:
        let register_store_address = 200;
        let expected_register_result = 0x49;

        // 3. Indirect store: Original value (A=0x41) = 0x41
        let indirect_result_address = *labels.get("INDIRECT_RESULT").unwrap();
        let expected_indirect_result = 0x49;

        tracing::debug!(
            pc = format!("0x{:04X}", machine_state.pc.get()),
            expected_direct = format!("0x{:04X}", expected_direct_result),
            direct_addr = format!("0x{:04X}", direct_result_address),
            actual_direct = format!(
                "0x{:04X}",
                machine_state.memory[direct_result_address as usize].get()
            ),
            expected_register = format!("0x{:04X}", expected_register_result),
            register_addr = format!("0x{:04X}", register_store_address),
            actual_register = format!(
                "0x{:04X}",
                machine_state.memory[register_store_address].get()
            ),
            expected_indirect = format!("0x{:04X}", expected_indirect_result),
            indirect_addr = format!("0x{:04X}", indirect_result_address),
            actual_indirect = format!(
                "0x{:04X}",
                machine_state.memory[indirect_result_address as usize].get()
            ),
            "Final machine state"
        );

        // Verify direct storage result
        assert_eq!(
            machine_state.memory[direct_result_address as usize].get(),
            expected_direct_result,
            "Direct result should be stored correctly in memory"
        );

        // Verify register-based store result
        assert_eq!(
            machine_state.memory[register_store_address].get(),
            expected_register_result,
            "Register-based result should be stored correctly in memory"
        );

        // Verify indirect store result
        assert_eq!(
            machine_state.memory[indirect_result_address as usize].get(),
            expected_indirect_result,
            "Indirect result should be stored correctly in memory"
        );

        // Verify the output
        assert_eq!(
            machine_state.output.len(),
            1,
            "Program should have produced one output"
        );
        assert_eq!(
            machine_state.output.chars().next(),
            Some('I'),
            "Output should match the final value of R0"
        );

        tracing::info!("Comprehensive program execution test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_cell_index_and_range() {
    tracing::info_span!("test_cell_index_and_range").in_scope(|| {
        tracing::info!("Starting test for BitAddressable index and range");

        // Test the index function
        let test_cell = EmulatorCell(0b1010_1100_0011_0101); // 0xAC35

        tracing::debug!(
            cell_value = format!("0x{:04X}", test_cell.get()),
            binary = format!("0b{:016b}", test_cell.get()),
            "Testing index operation on test cell"
        );

        // Test individual bit extraction
        let bit15 = test_cell.index(15).get();
        let bit10 = test_cell.index(10).get();
        let bit5 = test_cell.index(5).get();
        let bit0 = test_cell.index(0).get();

        tracing::debug!(
            bit15 = bit15,
            bit10 = bit10,
            bit5 = bit5,
            bit0 = bit0,
            "Individual bit extraction results"
        );

        // Expected values from binary 1010110000110101
        assert_eq!(bit15, 1, "Bit 15 should be 1");
        assert_eq!(bit10, 1, "Bit 10 should be 1");
        assert_eq!(bit5, 1, "Bit 5 should be 1");
        assert_eq!(bit0, 1, "Bit 0 should be 1");

        // Test the range function
        tracing::debug!(
            cell_value = format!("0x{:04X}", test_cell.get()),
            "Testing range operation on test cell"
        );

        // Extract different ranges
        let range_15_12 = test_cell.range(15..12).get(); // 1010
        let range_11_8 = test_cell.range(11..8).get(); // 1100
        let range_7_4 = test_cell.range(7..4).get(); // 0011
        let range_3_0 = test_cell.range(3..0).get(); // 0101

        tracing::debug!(
            range_15_12 = format!("0x{:X}", range_15_12),
            range_11_8 = format!("0x{:X}", range_11_8),
            range_7_4 = format!("0x{:X}", range_7_4),
            range_3_0 = format!("0x{:X}", range_3_0),
            "Range extraction results"
        );

        assert_eq!(range_15_12, 0xA, "Range 15..12 should be 0xA");
        assert_eq!(range_11_8, 0xC, "Range 11..8 should be 0xC");
        assert_eq!(range_7_4, 0x3, "Range 7..4 should be 0x3");
        assert_eq!(range_3_0, 0x5, "Range 3..0 should be 0x5");

        // Test sign extension with range and sext
        let negative_value = test_cell.range(15..12).sext(3).get(); // Sign-extend 1010
        let positive_value = test_cell.range(3..0).sext(3).get(); // Sign-extend 0101

        tracing::debug!(
            negative_range = format!("0b{:04b}", test_cell.range(15..12).get()),
            negative_extended = format!("0x{:04X}", negative_value),
            positive_range = format!("0b{:04b}", test_cell.range(3..0).get()),
            positive_extended = format!("0x{:04X}", positive_value),
            "Sign extension results"
        );

        // 1010 sign-extended from bit 3 should have 1s in upper bits
        assert_eq!(
            negative_value, 0xFFFA,
            "Negative value sign-extended from bit 3 should be 0xFFFA"
        );
        // 0101 sign-extended from bit 3 should keep 0s in upper bits
        assert_eq!(
            positive_value, 0x0005,
            "Positive value sign-extended from bit 3 should be 0x0005"
        );

        tracing::info!("BitAddressable index and range test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_c_println_assembly() {
    tracing::info_span!("test_c_println_assembly").in_scope(|| {
        tracing::info!("Starting test for C-generated assembly code");

        // Load assembly file
        let assembly_content = include_str!("../../c-println.asm");

        tracing::debug!(
            assembly_size = assembly_content.len(),
            "Loaded assembly file from c-println.asm"
        );

        // Parse the program
        tracing::debug!("Parsing C-generated assembly program");
        let parse_result = Emulator::parse_program(assembly_content);

        // Check if parsing was successful
        assert!(parse_result.is_ok(), "Assembly parsing should succeed");

        let ParseOutput {
            machine_code,
            line_to_address,
            labels,
            orig_address,
        } = parse_result.unwrap();

        tracing::debug!(
            instruction_count = machine_code.len(),
            label_count = labels.len(),
            origin = format!("0x{:04X}", orig_address),
            "Assembly parsed successfully"
        );

        // Create an emulator and load the program
        let mut machine_state = Emulator::new();
        tracing::debug!("Loading assembly program into emulator");
        machine_state.flash_memory(machine_code, orig_address);

        // Execute the program with a maximum number of steps
        tracing::debug!("Beginning C-generated assembly program execution");
        let max_steps = 10000; // Prevent infinite loops
        machine_state.running = true;
        let result = machine_state.run(Some(max_steps));

        // Verify execution completed successfully
        assert!(result.is_ok(), "Assembly execution should succeed");

        // Verify the machine halted
        assert!(!machine_state.running, "Machine should have halted");

        // Log the output
        tracing::debug!(output = machine_state.output, "Assembly program output");

        // Verify some output was produced
        assert!(
            !machine_state.output.is_empty(),
            "Assembly program should produce output"
        );

        tracing::info!("C-generated assembly test completed successfully");
    });
}
