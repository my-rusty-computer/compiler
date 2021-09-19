#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod printer;

#[derive(PartialEq, Debug)]
pub enum Operation {
    // Data transfer
    Mov,
    Push,
    Pop,
    Xchg,
    In,
    Out,
    Xlat,
    Lea,
    Lds,
    Les,
    Lahf,
    Sahf,
    Pushf,
    Popf,

    // Arithmetic
    Add,
    Adc,
    Inc,
    Aaa,
    Baa,
    Sub,
    Sbb,
    Dec,
    Neg,
    Cmp,
    Aas,
    Das,
    Mul,
    Imul,
    Aam,
    Div,
    Idiv,
    Aad,
    Cbw,
    Cwd,

    // Logic
    Not,
    Shl,
    Shr,
    Sar,
    Rol,
    Ror,
    Rcl,
    Rcr,
    And,
    Test,
    Or,
    Xor,

    // String manipulation
    // Rep,
    Cmpsb,
    Cmpsw,
    Lodsb,
    Lodsw,
    Movsb,
    Movsw,
    Scasb,
    Scasw,
    Stosb,
    Stosw,

    // Control transfer
    Call,
    Jmp,
    Ret,
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jnle,
    Jnb,
    Jnbe,
    Jnp,
    Jno,
    Jns,
    Loop,
    Loopz,
    Loopnz,
    Jcxz,
    Int,
    Into,
    IRet,

    // Processor control
    Clc,
    Cmc,
    Stc,
    Cld,
    Std,
    Cli,
    Sti,
    Hlt,
    Wait,
    Esc,
    Lock,
}

#[derive(PartialEq, Debug)]
pub enum Register {
    AlAx,
    ClCx,
    DlDx,
    BlBx,
    AhSp,
    ChBp,
    DhSi,
    BhDi,
}

#[derive(PartialEq, Debug)]
pub enum Segment {
    Es,
    Cs,
    Ss,
    Ds,
}

#[derive(Debug, PartialEq)]
pub enum AddressingMode {
    BxSi,
    BxDi,
    BpSi,
    BpDi,
    Si,
    Di,
    Bp,
    Bx,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum OperandSize {
    Byte,
    Word,
}

#[derive(PartialEq, Debug)]
pub enum OperandType {
    Direct(u16),
    Indirect(AddressingMode, u16),
    Register(Register),
    Segment(Segment),
    Immediate(u16),
}

#[derive(PartialEq, Debug)]
pub struct Operand(pub OperandType, pub OperandSize);

#[derive(PartialEq, Debug)]
pub enum OperandSet {
    None,
    Destination(Operand),
    DestinationAndSource(Operand, Operand),
    Offset(u16),
    SegmentAndOffset(u16, u16),
}

#[derive(PartialEq, Debug)]
pub enum Repeat {
    Equal,
    NotEqual,
}

#[derive(PartialEq, Debug)]
pub struct Instruction {
    pub operation: Operation,
    pub segment_override: Option<Segment>,
    pub repeat: Option<Repeat>,
    pub operands: OperandSet,
}

impl Instruction {
    pub fn new(operation: Operation, operands: OperandSet) -> Self {
        Self {
            operation,
            segment_override: None,
            repeat: None,
            operands,
        }
    }
}
