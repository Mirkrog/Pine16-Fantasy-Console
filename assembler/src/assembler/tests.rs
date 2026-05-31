/* ////////////////////////////////////
    AI generated Unit Tests
//////////////////////////////////// */

use super::*;

// --- Unit Tests for Parsing ---

#[test]
fn test_parse_register() {
    assert_eq!(Register::parse_register("A").unwrap(), Register::A);
    assert_eq!(Register::parse_register("D").unwrap(), Register::D);
    assert!(Register::parse_register("X").is_err());
}

#[test]
fn test_parse_argument_types() {
    // Direct Value ($)
    let arg = Argument::parse_argument("$42").unwrap();
    assert_eq!(arg, Argument::DirectValue(42));
    assert!(arg.is_direct_value());

    // Address (@)
    let arg = Argument::parse_argument("@256").unwrap();
    assert_eq!(arg, Argument::Address(256));
    assert!(arg.is_address());

    // Register (*)
    let arg = Argument::parse_argument("*B").unwrap();
    assert_eq!(arg, Argument::Register(Register::B));
    assert!(arg.is_register());

    // Label
    let arg = Argument::parse_argument("loopstart").unwrap();
    assert_eq!(arg, Argument::Label("loopstart".to_string()));
    assert!(arg.is_label());
}

#[test]
fn test_parse_argument_errors() {
    assert!(Argument::parse_argument("$").is_err()); // Missing numerical value
    assert!(Argument::parse_argument("@abc").is_err()); // Invalid numeric address
    assert!(Argument::parse_argument("*X").is_err()); // Invalid register string
    assert!(Argument::parse_argument("label_with_numbers123").is_err()); // Non-alphabetic label
}

// --- Integration Tests for Assembly ---

#[test]
fn test_assemble_basic_instructions() {
    let source = "mov *A $10";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1585, 0, 10]);
}

#[test]
fn test_assemble_with_comments_and_whitespace() {
    let source = "
            ; This is a comment at the start
            mov *B @50    ; Move value at address 50 to register B
            
            stall         ; Empty/Stall cycles
        ";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1586, 1, 50, 1280, 0, 0]);
}

#[test]
fn test_labels_and_jumps() {
    let source = "
            mov *A $5
            start:
            jmp start
        ";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    let expected_mov = vec![1585, 0, 5];
    let expected_jmp = vec![1856, 3, 0];

    let mut expected = expected_mov;
    expected.extend(expected_jmp);

    assert_eq!(result, expected);
    assert_eq!(*assembler.labels.get("start").unwrap(), 3);
}

// --- Error and Edge Case Variant Tests ---

#[test]
fn test_duplicate_labels_error() {
    let source = "
            start:
            mov *A $1
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
    let source = "add $10 *A";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("First argument is of invalid type"));
}

#[test]
fn test_missing_label_error() {
    let source = "jmp nonexistentlabel";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Label not found"));
}

#[test]
fn test_u16_overflow_error() {
    let source = "mov *A $65536";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
}
