/* ////////////////////////////////////
    AI generated Unit Tests
//////////////////////////////////// */

use super::*;

// --- Unit Tests for Parsing ---

#[test]
fn test_parse_register() {
    assert_eq!(Register::parse_register("A", 0).unwrap(), Register::A);
    assert_eq!(Register::parse_register("D", 0).unwrap(), Register::D);
    assert!(Register::parse_register("X", 0).is_err());
}

#[test]
fn test_parse_argument_types() {
    // Direct Value (#)
    let arg = Argument::parse_argument("#42", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Immediate(42));
    assert!(arg.is_immediate());

    // Address ($#)
    let arg = Argument::parse_argument("$#256", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Address(256));
    assert!(arg.is_address());

    // Register Pointed Address ($*)
    let arg = Argument::parse_argument("$*B", 0).unwrap();
    assert_eq!(
        arg.arg_type,
        ArgumentType::RegisterPointedAddress(Register::B)
    );

    // Register (*)
    let arg = Argument::parse_argument("*B", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Register(Register::B));
    assert!(arg.is_register());

    // Label
    let arg = Argument::parse_argument("loopstart", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Label("loopstart".to_string()));
    assert!(arg.is_label());
}

#[test]
fn test_parse_argument_errors() {
    assert!(Argument::parse_argument("#", 0).is_err()); // Missing numerical value
    assert!(Argument::parse_argument("$", 0).is_err()); // Expected identifier after $
    assert!(Argument::parse_argument("$#abc", 0).is_err()); // Invalid numeric address
    assert!(Argument::parse_argument("*X", 0).is_err()); // Invalid register string
}

// --- Integration Tests for Assembly ---

#[test]
fn test_assemble_basic_instructions() {
    // mov *A #10
    // OpCode::Mov = 6
    // arg0: Register (*A) -> Bytecode Type: 3, Value: 0
    // arg1: Immediate (#10) -> Bytecode Type: 1, Value: 10
    // First instruction (u16): (6 << 8) | (3 << 4) | 1 = 1536 | 48 | 1 = 1585
    let source = "mov *A #10";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1585, 0, 10]);
}

#[test]
fn test_assemble_with_comments_and_whitespace() {
    // mov *B $#50
    // OpCode::Mov = 6
    // arg0: Register (*B) -> Type: 3, Value: 1
    // arg1: Address ($#50) -> Type: 2, Value: 50
    // First instruction: (6 << 8) | (3 << 4) | 2 = 1536 | 48 | 2 = 1586
    //
    // stall
    // OpCode::Stall = 5
    // arg0: Empty -> Type: 0, Value: 0
    // arg1: Empty -> Type: 0, Value: 0
    // Fourth instruction: (5 << 8) | (0 << 4) | 0 = 1280
    let source = "
            ; This is a comment at the start
            mov *B $#50    ; Move value at address 50 to register B
            
            stall         ; Empty/Stall cycles
        ";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1586, 1, 50, 1280, 0, 0]);
}

#[test]
fn test_labels_and_jumps() {
    // mov *A #5  -> (6 << 8) | (3 << 4) | 1 = 1585, Val_A: 0, Val_5: 5
    // jmp start  -> OpCode::Jmp = 7. arg0: Label -> Type: 1, Value: address of start (1)
    //               arg1: Empty -> Type: 0, Value: 0
    //               Instruction: (7 << 8) | (1 << 4) | 0 = 1792 | 16 | 0 = 1808
    let source = "mov *A #5\nstart:\njmp start";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    let expected_mov = vec![1585, 0, 5];
    let expected_jmp = vec![1808, 1, 1]; // Points to instruction index 1

    let mut expected = expected_mov;
    expected.extend(expected_jmp);

    assert_eq!(result, expected);
    assert_eq!(*assembler.labels.get("start").unwrap(), 1);
}

// --- Error and Edge Case Variant Tests ---

#[test]
fn test_duplicate_labels_error() {
    let source = "
            start:
            mov *A #1
            start:
        ";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("defined twice"));
}

#[test]
fn test_invalid_argument_type_error() {
    let source = "add #10 *A";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // Validates against our updated Miette / AssemblerError debug print out string formatting
    assert!(err_msg.contains("InvalidArgumentType") || err_msg.contains("invalid_argument"));
}

#[test]
fn test_missing_label_error() {
    let source = "jmp nonexistentlabel";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("UnknownLabel") || err_msg.contains("unknown_label"));
}

#[test]
fn test_u16_overflow_error() {
    let source = "mov *A #65536";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
}
