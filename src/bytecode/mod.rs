use std::collections::HashMap;

mod compiler;
mod executor;

pub use compiler::BytecodeCompiler;
pub use executor::{BytecodeExecutor, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Bytecode {
    // Stack Operations
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushString(String),
    PushArrayF64(Vec<f64>),
    PushMap(HashMap<String, Value>),

    // Arithmetic Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Logical Operations
    And,
    Or,
    Not,

    // Comparison Operations
    Eq, // ==
    Ne, // !=
    Gt, // >
    Ge, // >=
    Lt, // <
    Le, // <=

    // Function Calls
    Call(String, usize), // Function name, argument count

    // Property Access
    GetProperty(String), // Access struct fields (e.g., "price.high")

    // Variable Handling
    LoadVariable(String),
    StoreVariable(String),

    // Control Flow
    Jump(usize),        // Jump to instruction index
    JumpIfTrue(usize),  // Jump if top of stack is true
    JumpIfFalse(usize), // Jump if top of stack is false
    Return,             // Return from function

    // Debugging / No-op
    NoOp,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    // Stack Operations
    PushInt = 0x01,
    PushFloat = 0x02,
    PushBool = 0x03,
    PushString = 0x04,
    PushArrayF64 = 0x05,
    PushMap = 0x06,

    // Arithmetic
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Mod = 0x14,
    Pow = 0x15,

    // Logical
    And = 0x20,
    Or = 0x21,
    Not = 0x22,

    // Comparisons
    Eq = 0x30,
    Ne = 0x31,
    Gt = 0x32,
    Ge = 0x33,
    Lt = 0x34,
    Le = 0x35,

    // Function Calls
    Call = 0x40,

    // Property Access
    GetProperty = 0x50,

    // Variable Handling
    LoadVariable = 0x60,
    StoreVariable = 0x61,

    // Control Flow
    Jump = 0x70,
    JumpIfTrue = 0x71,
    JumpIfFalse = 0x72,
    Return = 0x73,

    // No-op
    NoOp = 0xFF,
}

impl OpCode {
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::PushInt),
            0x02 => Some(Self::PushFloat),
            0x03 => Some(Self::PushBool),
            0x04 => Some(Self::PushString),
            0x05 => Some(Self::PushArrayF64),
            0x06 => Some(Self::PushMap),
            0x10 => Some(Self::Add),
            0x11 => Some(Self::Sub),
            0x12 => Some(Self::Mul),
            0x13 => Some(Self::Div),
            0x14 => Some(Self::Mod),
            0x15 => Some(Self::Pow),
            0x20 => Some(Self::And),
            0x21 => Some(Self::Or),
            0x22 => Some(Self::Not),
            0x30 => Some(Self::Eq),
            0x31 => Some(Self::Ne),
            0x32 => Some(Self::Gt),
            0x33 => Some(Self::Ge),
            0x34 => Some(Self::Lt),
            0x35 => Some(Self::Le),
            0x40 => Some(Self::Call),
            0x50 => Some(Self::GetProperty),
            0x60 => Some(Self::LoadVariable),
            0x61 => Some(Self::StoreVariable),
            0x70 => Some(Self::Jump),
            0x71 => Some(Self::JumpIfTrue),
            0x72 => Some(Self::JumpIfFalse),
            0x73 => Some(Self::Return),
            0xFF => Some(Self::NoOp),
            _ => None,
        }
    }
}
