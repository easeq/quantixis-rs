use crate::bytecode::OpCode;

pub struct BytecodeDecoder<'a> {
    bytecode: &'a [u8],
    position: usize,
}

impl<'a> BytecodeDecoder<'a> {
    pub fn new(bytecode: &'a [u8]) -> Self {
        Self {
            bytecode,
            position: 0,
        }
    }

    pub fn next_opcode(&mut self) -> Option<OpCode> {
        if self.position >= self.bytecode.len() {
            return None;
        }
        let opcode = self.bytecode[self.position];
        self.position += 1;
        OpCode::from_u8(opcode)
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.position >= self.bytecode.len() {
            return None;
        }
        let value = self.bytecode[self.position];
        self.position += 1;
        Some(value)
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        if self.position + 8 > self.bytecode.len() {
            return None;
        }
        let bytes: [u8; 8] = self.bytecode[self.position..self.position + 8]
            .try_into()
            .ok()?;
        self.position += 8;
        Some(i64::from_le_bytes(bytes))
    }
}
