use super::*;
use parse::ParseOutput;
use tracing_test::traced_test;

#[traced_test]
#[test]
fn test_full_program_execution() {
    tracing::info_span!("test_full_program_execution").in_scope(|| {
        tracing::info!("Starting comprehensive program execution test with all instructions");

        let program = r#"
            .ORIG x3000

            BR CODE
            
            ; First subroutine
            SUBROUTINE
                ADD R0, R0, #5      ; Add 5 to R0
                RET                 ; Return using R7

            ; Initialize registers
            CODE AND R0, R0, #0      ; Clear R0
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

            LD R2, REGISTER_STORE_ADDRESS ; store at x4000
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
            REGISTER_STORE_ADDRESS: .FILL x4000


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

        let mut machine_state = Emulator::new();

        // Parse the program
        tracing::debug!("Parsing program");
        let parse_result = Emulator::parse_program(program, None);

        // Check if parsing was successful
        assert!(parse_result.is_ok(), "Program parsing should succeed");

        let ParseOutput {
            labels,
            orig_address,
            machine_code,
            ..
        } = parse_result.unwrap();

        tracing::debug!("Loading program into emulator");
        machine_state.flash_memory(machine_code, orig_address);

        // Verify the program was loaded correctly
        assert_eq!(
            machine_state.pc.get(),
            0x0200,
            "PC should be set to OS origin address"
        );

        // Execute the program with a maximum number of steps
        tracing::debug!("Beginning program execution");
        let max_steps = 500; // Prevent infinite loops
        machine_state.start_running();
        let result = machine_state.run(Some(max_steps));

        tracing::debug!("Program execution completed");
        tracing::debug!("Result: {:?}", result);

        // Verify execution completed successfully
        assert!(result.is_ok(), "Program execution should succeed");

        // Verify the machine halted
        assert!(!machine_state.running(), "Machine should have halted");

        // Verify the results in memory
        let direct_result_address = *labels.get("RESULT").unwrap();
        let expected_direct_result = 0xFFB6;

        // 2. Register-based store:
        let register_store_address = 0x4000;
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

        tracing::info!("Comprehensive program execution test completed successfully");
    });
}

#[traced_test]
#[test]
fn test_cell_index_and_range() {
    tracing::info_span!("test_cell_index_and_range").in_scope(|| {
        tracing::info!("Starting test for BitAddressable index and range");

        // Test the index function
        let test_cell = EmulatorCell::new(0b1010_1100_0011_0101); // 0xAC35

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
        let assembly_content = include_str!("../../asm_tests/c-println.asm");

        tracing::debug!(
            assembly_size = assembly_content.len(),
            "Loaded assembly file from c-println.asm"
        );

        // Parse the program
        tracing::debug!("Parsing C-generated assembly program");
        let parse_result = Emulator::parse_program(assembly_content, None);

        // Check if parsing was successful
        assert!(parse_result.is_ok(), "Assembly parsing should succeed");

        let ParseOutput {
            machine_code,
            labels,
            orig_address,
            ..
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
        machine_state.start_running();
        let result = machine_state.run(Some(max_steps));

        tracing::debug!("Program execution completed");
        tracing::debug!("Result: {:?}", result);

        // Verify execution completed successfully
        assert!(result.is_ok(), "Assembly execution should succeed");

        // Verify the machine halted
        assert!(!machine_state.running(), "Machine should have halted");

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

// Individual Opcode Tests

fn run_instruction_test(
    initial_pc: u16,
    instruction: u16,
    setup_fn: impl FnOnce(&mut Emulator),
    assert_fn: impl FnOnce(&Emulator),
) {
    let mut machine = Emulator::new();
    machine.pc.set(initial_pc);
    machine.memory[initial_pc as usize].set(instruction);

    // Apply initial setup
    setup_fn(&mut machine);

    // Run the single instruction step
    machine.step();
    assert!(
        machine.exception.is_none(),
        "Instruction step failed: {:?}",
        machine.exception
    );

    // Assert final state
    assert_fn(&machine);
}
#[traced_test]
#[test]
fn test_add_register() {
    run_instruction_test(
        0x3000,
        0b0001_001_010_0_00_011, // ADD R1, R2, R3
        |m| {
            m.r[2].set(5);
            m.r[3].set(10);
        },
        |m| {
            assert_eq!(m.r[1].get(), 15, "R1 should be 5 + 10 = 15");
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (n, z, p) = m.get_nzp();
            assert!(p, "Positive flag should be set");
            assert!(!z, "Zero flag should be clear");
            assert!(!n, "Negative flag should be clear");
        },
    );
}

#[traced_test]
#[test]
fn test_add_immediate() {
    run_instruction_test(
        0x3000,
        0b0001_001_010_1_00101, // ADD R1, R2, #5
        |m| {
            m.r[2].set(10);
        },
        |m| {
            assert_eq!(m.r[1].get(), 15, "R1 should be 10 + 5 = 15");
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (.., p) = m.get_nzp();
            assert!(p, "Positive flag should be set");
        },
    );
}

#[traced_test]
#[test]
fn test_add_immediate_negative() {
    run_instruction_test(
        0x3000,
        0b0001_001_010_1_11111, // ADD R1, R2, #-1
        |m| {
            m.r[2].set(5);
        },
        |m| {
            assert_eq!(m.r[1].get(), 4, "R1 should be 5 + (-1) = 4");
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (.., p) = m.get_nzp();
            assert!(p, "Positive flag should be set");
        },
    );
}

#[traced_test]
#[test]
fn test_and_register() {
    run_instruction_test(
        0x3000,
        0b0101_001_010_0_00_011, // AND R1, R2, R3
        |m| {
            m.r[2].set(0b1100);
            m.r[3].set(0b1010);
        },
        |m| {
            assert_eq!(
                m.r[1].get(),
                0b1000,
                "R1 should be 0b1100 & 0b1010 = 0b1000"
            );
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (.., p) = m.get_nzp();
            assert!(p, "Positive flag should be set");
        },
    );
}

#[traced_test]
#[test]
fn test_and_immediate() {
    run_instruction_test(
        0x3000,
        0b0101_001_010_1_01010, // AND R1, R2, #10 (0b01010)
        |m| {
            m.r[2].set(0b1111); // 15
        },
        |m| {
            assert_eq!(m.r[1].get(), 0b01010, "R1 should be 0b1111 & 0b01010 = 10");
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (.., p) = m.get_nzp();
            assert!(p, "Positive flag should be set");
        },
    );
}

#[traced_test]
#[test]
fn test_br_taken() {
    run_instruction_test(
        0x3000,
        0b0000_0_0_1_000001010, // BRP +10 (PC -> 0x300B)
        |m| {
            // Set positive flag in PSR
            m.set_p();
        },
        |m| {
            assert_eq!(m.pc.get(), 0x300B, "PC should jump to 0x3000 + 1 + 10");
        },
    );
}

#[traced_test]
#[test]
fn test_br_not_taken() {
    run_instruction_test(
        0x3000,
        0b0000_1_0_0_000001010, // BRn +10
        |m| {
            // Set positive flag (condition doesn't match)
            m.set_p()
        },
        |m| {
            assert_eq!(m.pc.get(), 0x3001, "PC should only increment");
        },
    );
}

#[traced_test]
#[test]
fn test_jmp() {
    run_instruction_test(
        0x3000,
        0b1100_000_011_000000, // JMP R3
        |m| {
            m.r[3].set(0x4000); // Set target address in R3
        },
        |m| {
            assert_eq!(m.pc.get(), 0x4000, "PC should jump to address in R3");
        },
    );
}

#[traced_test]
#[test]
fn test_ret() {
    run_instruction_test(
        0x3000,
        0b1100_000_111_000000, // RET (JMP R7)
        |m| {
            m.r[7].set(0x5000); // Set return address in R7
        },
        |m| {
            assert_eq!(m.pc.get(), 0x5000, "PC should jump to address in R7");
        },
    );
}

#[traced_test]
#[test]
fn test_jsr() {
    run_instruction_test(
        0x3000,
        0b0100_1_00000010000, // JSR +16 (Target 0x3011)
        |_| {},               // No specific setup needed
        |m| {
            assert_eq!(
                m.r[7].get(),
                0x3001,
                "R7 should contain the return address (PC+1)"
            );
            assert_eq!(
                m.pc.get(),
                0x3011,
                "PC should jump to 0x3000 + 1 + 16 = 0x3011"
            );
        },
    );
}

#[traced_test]
#[test]
fn test_jsrr() {
    run_instruction_test(
        0x3000,
        0b0100_0_00_011_000000, // JSRR R3
        |m| {
            m.r[3].set(0x6000); // Set subroutine address in R3
        },
        |m| {
            assert_eq!(
                m.r[7].get(),
                0x3001,
                "R7 should contain the return address (PC+1)"
            );
            assert_eq!(m.pc.get(), 0x6000, "PC should jump to address in R3");
        },
    );
}

#[traced_test]
#[test]
fn test_ld() {
    run_instruction_test(
        0x3000,
        0b0010_001_000000101, // LD R1, +5 (Load from 0x3006)
        |m| {
            m.memory[0x3006].set(0xABCD); // Value to load
        },
        |m| {
            assert_eq!(m.r[1].get(), 0xABCD, "R1 should contain value from memory");
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (n, ..) = m.get_nzp();
            assert!(n, "Negative flag should be set (0xABCD)");
        },
    );
}

#[traced_test]
#[test]
fn test_ldr() {
    run_instruction_test(
        0x3000,
        0b0110_001_010_000101, // LDR R1, R2, #5
        |m| {
            m.r[2].set(0x4000); // Base address
            m.memory[0x4005].set(0x1234); // Value to load (0x4000 + 5)
        },
        |m| {
            assert_eq!(m.r[1].get(), 0x1234, "R1 should contain value from memory");
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (.., p) = m.get_nzp();
            assert!(p, "Positive flag should be set (0x1234)");
        },
    );
}

#[traced_test]
#[test]
fn test_lea() {
    run_instruction_test(
        0x3000,
        0b1110_001_000000101, // LEA R1, +5 (Load address 0x3006)
        |_| {},
        |m| {
            assert_eq!(
                m.r[1].get(),
                0x3006,
                "R1 should contain the effective address 0x3000 + 1 + 5"
            );
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
        },
    );
}

#[traced_test]
#[test]
fn test_not() {
    run_instruction_test(
        0x3000,
        0b1001_001_010_111111, // NOT R1, R2
        |m| {
            m.r[2].set(0b0000_1111_0000_1111); // Input value
        },
        |m| {
            assert_eq!(
                m.r[1].get(),
                0b1111_0000_1111_0000,
                "R1 should contain the bitwise NOT"
            );
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
            let (n, ..) = m.get_nzp();
            assert!(n, "Negative flag should be set");
        },
    );
}

#[traced_test]
#[test]
fn test_st() {
    run_instruction_test(
        0x3000,
        0b0011_001_000000101, // ST R1, +5 (Store to 0x3006)
        |m| {
            m.r[1].set(0xFACE); // Value to store
        },
        |m| {
            assert_eq!(
                m.memory[0x3006].get(),
                0xFACE,
                "Memory at 0x3006 should contain value from R1"
            );
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
        },
    );
}

#[traced_test]
#[test]
fn test_str() {
    run_instruction_test(
        0x3000,
        0b0111_001_010_000101, // STR R1, R2, #5
        |m| {
            m.r[1].set(0xBEEF); // Value to store
            m.r[2].set(0x4000); // Base address
        },
        |m| {
            assert_eq!(
                m.memory[0x4005].get(),
                0xBEEF,
                "Memory at 0x4005 should contain value from R1"
            );
            assert_eq!(m.pc.get(), 0x3001, "PC should increment");
        },
    );
}
