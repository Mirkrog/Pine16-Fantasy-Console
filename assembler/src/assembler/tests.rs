/* ////////////////////////////////////
    AI generated Unit Tests
//////////////////////////////////// */

use super::*;

#[test]
fn test_parse_register() {
    assert_eq!(Register::parse_register("A", 0).unwrap(), Register::A);
    assert_eq!(Register::parse_register("D", 0).unwrap(), Register::D);
    assert!(Register::parse_register("X", 0).is_err());
}

#[test]
fn test_parse_argument_types() {
    let arg = Argument::parse_argument("#42", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Immediate(42));
    assert!(arg.is_immediate());

    let arg = Argument::parse_argument("$#256", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Address(256));
    assert!(arg.is_address());

    let arg = Argument::parse_argument("$*B", 0).unwrap();
    assert_eq!(
        arg.arg_type,
        ArgumentType::RegisterPointedAddress(Register::B)
    );

    let arg = Argument::parse_argument("*B", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Register(Register::B));
    assert!(arg.is_register());

    let arg = Argument::parse_argument("loopstart", 0).unwrap();
    assert_eq!(arg.arg_type, ArgumentType::Label("loopstart".to_string()));
    assert!(arg.is_label());
}

#[test]
fn test_parse_argument_errors() {
    assert!(Argument::parse_argument("#invalid", 0).is_err());
    assert!(Argument::parse_argument("$invalid", 0).is_err());
    assert!(Argument::parse_argument("$#abc", 0).is_err());
    assert!(Argument::parse_argument("*X", 0).is_err());
}

#[test]
fn test_assemble_basic_instructions() {
    let source = "mov *A #10";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1585, 0, 10]);
}

#[test]
fn test_assemble_with_comments_and_whitespace() {
    let source = "
            ; This is a comment at the start
            mov *B $#50    ; Move value at address 50 to register B
            
            mov *A #10     ; Dummy instruction to test spacing
        ";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1586, 1, 50, 1585, 0, 10]);
}

#[test]
fn test_labels_and_jumps() {
    let source = "mov *A #5\nstart:\njmp start";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble().unwrap();

    assert_eq!(result, vec![1585, 0, 5, 1808, 1, 0]);
}

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
}

#[test]
fn test_missing_label_error() {
    let source = "jmp nonexistentlabel";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("UnknownLabel")
            || err_msg.contains("unknown_label")
            || err_msg.contains("missing")
    );
}

#[test]
fn test_u16_overflow_error() {
    let source = "mov *A #65536";
    let mut assembler = Assembler::new(source);
    let result = assembler.assemble();

    assert!(result.is_err());
}
