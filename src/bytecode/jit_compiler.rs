use crate::bytecode::{BytecodeReader, Instruction, Value};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

/// JIT Compiler for Bytecode Execution
pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    module: JITModule,
    compiled_functions: HashMap<String, *const u8>,
}

impl JITCompiler {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .expect("JITBuilder creation failed");
        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            module,
            compiled_functions: HashMap::new(),
        }
    }

    /// Compile Bytecode to Native Code
    pub fn compile(&mut self, bytecode: &[u8]) -> Result<*const u8, String> {
        let mut ctx = self.module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        builder.seal_block(block);

        let mut reader = BytecodeReader::new(bytecode);
        let mut stack = Vec::new();
        let mut variable_map = HashMap::new();

        while let Some(instr) = reader.read_instruction() {
            match instr {
                Instruction::Push(Value::Number(n)) => {
                    let val = builder.ins().f64const(n);
                    stack.push(val);
                }
                Instruction::Add => {
                    let right = stack.pop().ok_or("Stack underflow")?;
                    let left = stack.pop().ok_or("Stack underflow")?;
                    let result = builder.ins().fadd(left, right);
                    stack.push(result);
                }
                Instruction::Sub => {
                    let right = stack.pop().ok_or("Stack underflow")?;
                    let left = stack.pop().ok_or("Stack underflow")?;
                    let result = builder.ins().fsub(left, right);
                    stack.push(result);
                }
                Instruction::Mul => {
                    let right = stack.pop().ok_or("Stack underflow")?;
                    let left = stack.pop().ok_or("Stack underflow")?;
                    let result = builder.ins().fmul(left, right);
                    stack.push(result);
                }
                Instruction::Div => {
                    let right = stack.pop().ok_or("Stack underflow")?;
                    let left = stack.pop().ok_or("Stack underflow")?;
                    let result = builder.ins().fdiv(left, right);
                    stack.push(result);
                }
                Instruction::CallFunction { name, args } => {
                    if let Some(&func_ptr) = self.compiled_functions.get(&name) {
                        let callee = builder.ins().iconst(types::I64, func_ptr as i64);
                        let args_vec: Vec<Value> = stack.drain(stack.len() - args..).collect();
                        let result = builder.ins().call_indirect(callee, &args_vec);
                        stack.push(result);
                    } else {
                        return Err(format!("Unknown function: {}", name));
                    }
                }
                _ => return Err("Unsupported instruction in JIT".to_string()),
            }
        }

        let return_val = stack.pop().ok_or("Empty stack at return")?;
        builder.ins().return_(&[return_val]);
        builder.finalize();

        let func_id = self
            .module
            .declare_function("jit_func", Linkage::Export, &ctx.func.signature)
            .map_err(|e| e.to_string())?;
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
        self.module.finalize_definitions();

        let func_ptr = self.module.get_finalized_function(func_id);
        Ok(func_ptr)
    }

    /// Execute compiled JIT function
    pub fn execute(&self, func_ptr: *const u8) -> f64 {
        let func: extern "C" fn() -> f64 = unsafe { std::mem::transmute(func_ptr) };
        func()
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_jit_execution() {
        let mut jit = JITCompiler::new();

        let mut writer = BytecodeWriter::new();
        writer.write_instruction(&Instruction::Push(Value::Number(5.0)));
        writer.write_instruction(&Instruction::Push(Value::Number(3.0)));
        writer.write_instruction(&Instruction::Add);

        let bytecode = writer.into_bytes();
        let func_ptr = jit.compile(&bytecode).expect("JIT Compilation failed");

        let result = jit.execute(func_ptr);
        assert!((result - 8.0).abs() < f64::EPSILON);
    }
}
