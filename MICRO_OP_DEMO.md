# Micro-Op Architecture Demo

This document demonstrates the new micro-op architecture that has been successfully implemented in the LC-3 emulator.

## Overview

The micro-op system transforms the emulator from a rigid 5-phase state machine into a flexible, instruction-driven execution model. Each instruction generates its own execution plan as a series of micro-operations that map to the textbook's 5-phase model for educational clarity.

## Key Benefits

1. **Educational Clarity**: Micro-ops use textbook notation like `ALU_OUT <- R1 + R2`
2. **Flexibility**: Different instructions can have different numbers of cycles
3. **Maintainability**: Clean separation between instruction logic and execution
4. **Debugging**: Each micro-op is explicit and traceable

## Architecture Components

### Core Data Structures

- `MicroOp`: Represents a single atomic CPU operation
- `DataSource`/`DataDestination`: Represents sources and destinations of data
- `MicroOpGenerator`: Trait for instructions that can generate execution plans

### The `micro_op!` Macro

The macro provides a clean DSL for writing micro-operations:

```rust
// Phase transitions
micro_op!(-> Execute)

// Data transfers  
micro_op!(R(1) <- R(2))        // Register to register
micro_op!(PC <- MAR)           // Component to component
micro_op!(MDR <- MEM[MAR])     // Memory read

// ALU operations
micro_op!(ALU_OUT <- R(1) + R(2))     // ADD registers
micro_op!(ALU_OUT <- R(1) + IMM(5))   // ADD immediate
micro_op!(ALU_OUT <- NOT R(1))        // NOT operation

// Flags and control
micro_op!(SET_CC(3))                  // Update condition codes
micro_op!(SET_FLAG(WriteMemory))      // Signal memory write
```

## Example: ADD Instruction

The ADD instruction now generates a clean, readable execution plan:

### Register Mode: `ADD R1, R2, R3`

```rust
vec![
    // Cycle 1: Execute
    vec![
        micro_op!(-> Execute),
        micro_op!(ALU_OUT <- R(2) + R(3)),
    ],
    // Cycle 2: Store Result  
    vec![
        micro_op!(-> StoreResult),
        micro_op!(R(1) <- AluOut),
        micro_op!(SET_CC(1)),
    ],
]
```

### Immediate Mode: `ADD R1, R2, #5`

```rust
vec![
    // Cycle 1: Execute
    vec![
        micro_op!(-> Execute),
        micro_op!(ALU_OUT <- R(2) + IMM(5)),
    ],
    // Cycle 2: Store Result
    vec![
        micro_op!(-> StoreResult), 
        micro_op!(R(1) <- AluOut),
        micro_op!(SET_CC(1)),
    ],
]
```

## Test Results

All micro-op tests are passing:

```
running 4 tests
test emulator::ops::add::tests::test_add_immediate_negative_micro_op_generation ... ok
test emulator::ops::add::tests::test_add_immediate_micro_op_generation ... ok
test emulator::ops::add::tests::test_add_display_format ... ok
test emulator::ops::add::tests::test_add_register_micro_op_generation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
```

## Display Output

Micro-ops render in clear, textbook-style notation:

- `R1 <- R2` (register transfer)
- `ALU_OUT <- R1 + R2` (ALU addition)
- `PC <- MAR` (component transfer)
- `-> Execute` (phase transition)
- `SET_CC(R3)` (condition code update)
- `[Reading from memory]` (informational message)

## Implementation Status

✅ Core micro-op data structures
✅ Macro DSL for readable micro-op creation
✅ ADD instruction fully converted
✅ Comprehensive test coverage
✅ Display formatting
✅ Educational phase mapping

## Next Steps

1. Update Emulator struct to support execution plans
2. Implement micro-op interpreter in main loop
3. Convert remaining instructions (AND, NOT, LD, ST, etc.)
4. Update UI to display micro-ops in CPU state pane
5. Add performance optimizations for faster execution modes

## Code Quality

- All clippy warnings resolved
- Proper error handling
- Comprehensive test coverage
- Clean separation of concerns
- Backward compatibility during transition

The micro-op architecture successfully provides a foundation for both educational clarity and implementation flexibility, making the emulator easier to understand, debug, and extend.