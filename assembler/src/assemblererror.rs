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
    #[diagnostic(
        code(assembler::too_many_arguments),
        help("the maximum amount of arguments for any opcode is 2")
    )]
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
    #[error("Misplaced comma")]
    #[diagnostic(
        code(assembler::misplaced_comma),
        help("Commas should only be used to seperate parts of the instruction")
    )]
    MisplacedComma {
        #[label("this comma is misplaced")]
        span: SourceSpan,
    },
    #[error("Useless Data Definition")]
    #[diagnostic(
        code(assembler::useless_data_definition),
        help("The consoles memory is cleared when a ROM is loaded")
    )]
    UselessDataDefinition {
        #[label("this data definition is useless")]
        span: SourceSpan,
    },
    #[error("Wrong Amount Of Arguments")]
    #[diagnostic(code(assembler::wrong_argument_amount))]
    WrongArgumentAmount {
        expected_amount: usize,
        amount: usize,
        #[label("expected {expected_amount} of argument(s), got {amount}")]
        span: SourceSpan,
    },
    #[error("Palette Index Out Of Bounds")]
    #[diagnostic(
        code(assembler::palette_index_out_of_bounds),
        help("The palette used by the console is only 4 bit")
    )]
    PaletteIndexOutOfBounds {
        number: u16,
        #[label("number out of range (0 - 15): {number}")]
        span: SourceSpan,
    },
    #[error("Data is initialized beyond RAM boundaries")]
    #[diagnostic(
        code(assembler::data_initialized_beyond_ram_bounds),
        help(
            "You are initializing memory addresses that go beyond the 16-bit integer limit through a too big data block or because the data block is repeated too often"
        )
    )]
    DataInitializedBeyondRAMBounds {
        address: usize,
        #[label("The Data is overflowing to address: {address}")]
        span: SourceSpan,
    },
    #[error("Unknown Data Operator")]
    #[diagnostic(
        code(assembler::unknown_data_operator),
        help(
            "use \"to\" to specify the location of the data and \"repeat\" to specify the length, both are not necessary you can also leave them out"
        )
    )]
    UnknownDataOperator {
        #[label("This data operator is unknown")]
        span: SourceSpan,
    },
    #[error("Number Infront Of Operation")]
    #[diagnostic(
        code(assembler::number_infront_of_operator),
        help(
            "Numbers can not be infront of operators/nan, because they would be parsed as data which would be misleading"
        )
    )]
    NumberInfrontOperation {
        #[label("This number is infront of an operator/nan")]
        span: SourceSpan,
    },
}
