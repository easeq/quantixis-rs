use crate::bytecode::{Bytecode, OpCode};

pub struct BytecodeEncoder;

impl BytecodeEncoder {
    pub fn encode(bytecode: &[Bytecode]) -> Vec<u8> {
        let mut output = Vec::new();
        for instruction in bytecode {
            match instruction {
                Bytecode::PushInt(val) => {
                    output.push(OpCode::PushInt as u8);
                    output.extend(&val.to_le_bytes());
                }
                Bytecode::PushFloat(val) => {
                    output.push(OpCode::PushFloat as u8);
                    output.extend(&val.to_le_bytes());
                }
                Bytecode::PushBool(val) => {
                    output.push(OpCode::PushBool as u8);
                    output.push(if *val { 1 } else { 0 });
                }
                Bytecode::PushString(val) => {
                    output.push(OpCode::PushString as u8);
                    output.extend((val.len() as u32).to_le_bytes());
                    output.extend(val.as_bytes());
                }
                // Bytecode::PushArrayF64(val) => {
                //     output.push(OpCode::PushArrayF64 as u8);
                //     output.extend((val.len() as u32).to_le_bytes());
                //     output.extend(val.as_bytes());
                // }
                Bytecode::Add => output.push(OpCode::Add as u8),
                Bytecode::Sub => output.push(OpCode::Sub as u8),
                Bytecode::Mul => output.push(OpCode::Mul as u8),
                Bytecode::Div => output.push(OpCode::Div as u8),
                Bytecode::Mod => output.push(OpCode::Mod as u8),
                Bytecode::And => output.push(OpCode::And as u8),
                Bytecode::Or => output.push(OpCode::Or as u8),
                Bytecode::Not => output.push(OpCode::Not as u8),
                Bytecode::Eq => output.push(OpCode::Eq as u8),
                Bytecode::Ne => output.push(OpCode::Ne as u8),
                Bytecode::Gt => output.push(OpCode::Gt as u8),
                Bytecode::Ge => output.push(OpCode::Ge as u8),
                Bytecode::Lt => output.push(OpCode::Lt as u8),
                Bytecode::Le => output.push(OpCode::Le as u8),
                Bytecode::Jump(addr) => {
                    output.push(OpCode::Jump as u8);
                    output.extend(&addr.to_le_bytes());
                }
                Bytecode::JumpIfTrue(addr) => {
                    output.push(OpCode::JumpIfTrue as u8);
                    output.extend(&addr.to_le_bytes());
                }
                Bytecode::JumpIfFalse(addr) => {
                    output.push(OpCode::JumpIfFalse as u8);
                    output.extend(&addr.to_le_bytes());
                }
                Bytecode::Return => output.push(OpCode::Return as u8),
                Bytecode::NoOp => output.push(OpCode::NoOp as u8),
                _ => unimplemented!("Encoding not implemented for {:?}", instruction),
            }
        }
        output
    }
}

// /// Bytecode Writer to serialize instructions into binary
// pub struct BytecodeWriter {
//     buffer: Vec<u8>,
// }
//
// impl BytecodeWriter {
//     pub fn new() -> Self {
//         Self { buffer: Vec::new() }
//     }
//
//     pub fn write_instruction(&mut self, instr: &Instruction) {
//         match instr {
//             Instruction::Push(value) => {
//                 self.buffer.push(0x01); // Opcode for Push
//                 self.write_value(value);
//             }
//             Instruction::Add => self.buffer.push(0x02), // Opcode for Add
//             Instruction::Sub => self.buffer.push(0x03), // Opcode for Sub
//             Instruction::Mul => self.buffer.push(0x04), // Opcode for Mul
//             Instruction::Div => self.buffer.push(0x05), // Opcode for Div
//             Instruction::Mod => self.buffer.push(0x06), // Opcode for Mod
//             Instruction::CallFunction { name, args } => {
//                 self.buffer.push(0x10); // Opcode for CallFunction
//                 self.write_string(name);
//                 self.buffer.push(*args as u8);
//             }
//             Instruction::PropertyAccess { property } => {
//                 self.buffer.push(0x11); // Opcode for PropertyAccess
//                 self.write_string(property);
//             }
//             _ => unimplemented!("Other instructions not yet implemented"),
//         }
//     }
//
//     fn write_value(&mut self, value: &Value) {
//         match value {
//             Value::Number(n) => {
//                 self.buffer.push(0x20); // Type code for Number
//                 self.buffer.extend_from_slice(&n.to_le_bytes());
//             }
//             Value::Boolean(b) => {
//                 self.buffer.push(0x21); // Type code for Boolean
//                 self.buffer.push(if *b { 1 } else { 0 });
//             }
//             Value::Identifier(id) => {
//                 self.buffer.push(0x22); // Type code for Identifier
//                 self.write_string(id);
//             }
//             Value::Array(arr) => {
//                 self.buffer.push(0x23); // Type code for Array
//                 self.buffer.push(arr.len() as u8);
//                 for num in arr {
//                     self.buffer.extend_from_slice(&num.to_le_bytes());
//                 }
//             }
//         }
//     }
//
//     fn write_string(&mut self, s: &str) {
//         let bytes = s.as_bytes();
//         self.buffer.push(bytes.len() as u8);
//         self.buffer.extend_from_slice(bytes);
//     }
//
//     pub fn into_bytes(self) -> Vec<u8> {
//         self.buffer
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_bytecode_encoding_decoding() {
//         let mut writer = BytecodeWriter::new();
//         let instrs = vec![
//             Instruction::Push(Value::Number(42.0)),
//             Instruction::Add,
//             Instruction::CallFunction {
//                 name: "sma".to_string(),
//                 args: 2,
//             },
//         ];
//
//         for instr in &instrs {
//             writer.write_instruction(instr);
//         }
//
//         let bytes = writer.into_bytes();
//         let mut reader = BytecodeReader::new(&bytes);
//
//         let mut decoded_instrs = Vec::new();
//         while let Some(instr) = reader.read_instruction() {
//             decoded_instrs.push(instr);
//         }
//
//         assert_eq!(instrs, decoded_instrs);
//     }
//
//     #[test]
//     fn test_bytecode_encoding_numbers() {
//         let mut writer = BytecodeWriter::new();
//         writer.write_instruction(&Instruction::Push(Value::Number(99.99)));
//
//         let bytes = writer.into_bytes();
//         let mut reader = BytecodeReader::new(&bytes);
//
//         if let Some(Instruction::Push(Value::Number(value))) = reader.read_instruction() {
//             assert!((value - 99.99).abs() < f64::EPSILON);
//         } else {
//             panic!("Decoding failed for number");
//         }
//     }
//
//     #[test]
//     fn test_bytecode_encoding_booleans() {
//         let mut writer = BytecodeWriter::new();
//         writer.write_instruction(&Instruction::Push(Value::Boolean(true)));
//
//         let bytes = writer.into_bytes();
//         let mut reader = BytecodeReader::new(&bytes);
//
//         if let Some(Instruction::Push(Value::Boolean(value))) = reader.read_instruction() {
//             assert_eq!(value, true);
//         } else {
//             panic!("Decoding failed for boolean");
//         }
//     }
//
//     #[test]
//     fn test_bytecode_encoding_function_call() {
//         let mut writer = BytecodeWriter::new();
//         writer.write_instruction(&Instruction::CallFunction {
//             name: "macd".to_string(),
//             args: 3,
//         });
//
//         let bytes = writer.into_bytes();
//         let mut reader = BytecodeReader::new(&bytes);
//
//         if let Some(Instruction::CallFunction { name, args }) = reader.read_instruction() {
//             assert_eq!(name, "macd");
//             assert_eq!(args, 3);
//         } else {
//             panic!("Decoding failed for function call");
//         }
//     }
// }
