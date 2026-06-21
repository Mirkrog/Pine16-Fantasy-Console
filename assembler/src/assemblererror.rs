use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum AssemblerError {
    #[error("Expected New Line")]
    #[diagnostic(
        code(assembler::expected_newline),
        help("You might have forgotten a `;`")
    )]
    ExpectedNewline {
        #[label("expected newline here")]
        span: SourceSpan,
    },
    #[error("Unknown OpCode: {opcode}")]
    #[diagnostic(code(assembler::unknown_opcode))]
    UnknownOpCode {
        opcode: String,
        #[label("this opcode is not recognized")]
        span: SourceSpan,
    },
    #[error("Unknown Argument Prefix: {argument_prefix}")]
    #[diagnostic(code(assembler::unknown_argument_type))]
    UnknownArgumentPrefix {
        argument_prefix: String,
        #[label("this argument prefix is not recognized")]
        span: SourceSpan,
    },
    #[error("Unknown Register: {register}")]
    #[diagnostic(code(assembler::unknown_register))]
    UnknownRegister {
        register: String,
        #[label("this register is not recognized")]
        span: SourceSpan,
    },
    #[error("Unknown Address Type: {address}")]
    #[diagnostic(code(assembler::unknown_address_type))]
    UnknownAddressType {
        address: String,
        #[label("this address type is not recognized")]
        span: SourceSpan,
    },
    #[error("Failed To Parse Number: {number}")]
    #[diagnostic(code(assembler::failed_to_parse_number))]
    FailedToParseNumber {
        number: String,
        parseerror: std::num::ParseIntError,
        #[label("failed to parse this number: {parseerror}")]
        span: SourceSpan,
    },
    #[error("Invalid Argument Type: {:?}", arg)]
    #[diagnostic(code(assembler::argument_has_wrong_type))]
    InvalidArgumentType {
        arg: crate::assembler::ArgumentType,
        #[label("this argument has the wrong type: {:?}", arg)]
        span: SourceSpan,
    },
    #[error("Unknown Label: {label}")]
    #[diagnostic(code(assembler::unknown_label))]
    UnknownLabel {
        label: String,
        #[label("this label was not recognized")]
        span: SourceSpan,
    },
    #[error("Too Many Arguments")]
    #[diagnostic(code(assembler::too_many_arguments))]
    TooManyArguments {
        #[label("too many arguments were given")]
        span: SourceSpan,
    },
    #[error("Missing Required Argument {argument_number}")]
    #[diagnostic(code(assembler::missing_required_argument))]
    MissingRequiredArgument {
        argument_number: usize,
        #[label("this opcode expects {argument_number} argument(s)")]
        span: SourceSpan,
    },
    #[error("Argument Missing Value")]
    #[diagnostic(code(assembler::argument_missing_value))]
    ArgumentMissingValue {
        #[label("this argument has no value")]
        span: SourceSpan,
    },
    #[error("Label Defined Multiple Times")]
    #[diagnostic(code(assembler::label_defined_multiple_times))]
    LabelDefinedMultipleTimes {
        #[label("here argument is defined first")]
        span: SourceSpan,
        #[label("here argument is defined again")]
        span1: SourceSpan,
    },
}
