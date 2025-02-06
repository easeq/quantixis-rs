use crate::bytecode::Bytecode;
use crate::jit::functions::func_pow;
use cranelift::codegen::ir::{FuncRef, GlobalValue, UserExternalNameRef};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::prelude::*;
use cranelift_jit::JITModule;
use cranelift_module::{DataId, FuncId, Module};
use log::{debug, trace};
use std::collections::HashMap;

pub struct JITCompiler {
    pub(super) module: JITModule,
    pub(super) functions_map: HashMap<String, FuncId>,
    pub(super) data_map: HashMap<String, DataId>,
    pub(super) stack: Vec<Value>,
}

impl JITCompiler {
    pub fn link_external_vars(
        &mut self,
        ctx: &mut codegen::Context,
    ) -> Result<HashMap<String, GlobalValue>, String> {
        let mut var_refs = HashMap::new();

        for (var_name, data_id) in self.data_map.iter() {
            let var_ref = self.module.declare_data_in_func(*data_id, &mut ctx.func);
            var_refs.insert(var_name.to_string(), var_ref);
        }

        Ok(var_refs)
    }

    pub fn link_external_functions(
        &mut self,
        ctx: &mut cranelift::codegen::Context,
    ) -> Result<HashMap<String, FuncRef>, String> {
        let mut func_refs = HashMap::new();
        for (func_name, func_id) in self.functions_map.clone().into_iter() {
            let func_ref = self.module.declare_func_in_func(func_id, &mut ctx.func);
            func_refs.insert(func_name, func_ref);
        }
        Ok(func_refs)
    }

    fn binary_op<F>(&mut self, builder: &mut FunctionBuilder, op: F) -> Result<(), String>
    where
        F: Fn(&mut FunctionBuilder, Value, Value) -> Value,
    {
        let (b, a) = (self.pop_value()?, self.pop_value()?);
        trace!("binary_op {a:?} {b:?}");
        let res = op(builder, a, b);
        self.stack.push(res);
        Ok(())
    }

    /// **Compiles bytecode to Cranelift IR**
    pub fn compile(&mut self, bytecode: &[Bytecode]) -> Result<(*const u8, *mut f64), String> {
        let pow_func_id = func_pow(&mut self.module)?;

        let mut ctx = self.module.make_context();
        ctx.func.signature.params.push(AbiParam::new(types::I64));
        ctx.func.signature.returns.push(AbiParam::new(types::F64));

        let func_refs = self.link_external_functions(&mut ctx)?;
        let var_refs = self.link_external_vars(&mut ctx)?;

        let pow_func_ref = self.module.declare_func_in_func(pow_func_id, &mut ctx.func);

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let main_block = builder.create_block();
        builder.switch_to_block(main_block);
        builder.append_block_params_for_function_params(main_block);

        let memory_ptr = builder.block_params(main_block)[0];
        // main block
        let variables = self.compile_main_block(
            &mut builder,
            &var_refs,
            &func_refs,
            bytecode,
            &pow_func_ref,
            memory_ptr,
        )?;
        builder.seal_block(main_block);

        let result = self.pop_value()?;
        builder.ins().return_(&[result]);
        builder.finalize();

        let func_id = self
            .module
            .declare_anonymous_function(&ctx.func.signature)
            .unwrap();
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let vars_ptr = if variables.len() > 0 {
            self.bind_variables(&variables)
        } else {
            std::ptr::null_mut()
        };

        let func = self.module.get_finalized_function(func_id);
        trace!("finalized function: {func:#?}");
        Ok((func as *const u8, vars_ptr))
    }

    /// Binds a list of variables dynamically, allocating a memory buffer.
    pub fn bind_variables(&mut self, var_names: &[String]) -> *mut f64 {
        let num_vars = var_names.len();
        let buffer = vec![0i64; num_vars].into_boxed_slice(); // Allocate space for variables
        Box::into_raw(buffer) as *mut f64
    }

    fn compile_main_block(
        &mut self,
        builder: &mut FunctionBuilder,
        var_refs: &HashMap<String, GlobalValue>,
        func_refs: &HashMap<String, FuncRef>,
        bytecode: &[Bytecode],
        pow_func_ref: &FuncRef,
        memory_ptr: Value,
    ) -> Result<Vec<String>, String> {
        let mut index = 0;
        let mut variables = vec![];
        for instruction in bytecode {
            match instruction {
                Bytecode::PushInt(value) => {
                    debug!("push int {value:?}");
                    let val = builder.ins().iconst(types::I64, *value);
                    self.stack.push(val);
                }
                Bytecode::PushFloat(value) => {
                    debug!("push float {value:?}");
                    let val = builder.ins().f64const(*value);
                    self.stack.push(val);
                }
                Bytecode::PushBool(value) => {
                    debug!("push bool {value:?}");
                    let val = builder.ins().f64const(*value as i8 as f64);
                    self.stack.push(val);
                }
                // Bytecode::PushArrayF64(values) => {
                //     for val in values {
                //         let val = builder.ins().f64const(*val);
                //         self.stack.push(val);
                //     }
                //     let array_count = builder.ins().iconst(types::I64, values.len() as i64);
                //     self.stack.push(array_count)
                // }
                Bytecode::Add => {
                    self.binary_op(builder, |builder, a, b| {
                        debug!("add {a:?} + {b:?}");
                        builder.ins().fadd(a, b)
                    })?;
                }
                Bytecode::Sub => {
                    self.binary_op(builder, |builder, a, b| {
                        debug!("sub {a:?} - {b:?}");
                        builder.ins().fsub(a, b)
                    })?;
                }
                Bytecode::Mul => {
                    self.binary_op(builder, |builder, a, b| {
                        debug!("mul {a:?} * {b:?}");
                        builder.ins().fmul(a, b)
                    })?;
                }
                Bytecode::Div => {
                    self.binary_op(builder, |builder, a, b| {
                        debug!("div {a:?} / {b:?}");
                        builder.ins().fdiv(a, b)
                    })?;
                }
                Bytecode::Mod => {
                    self.binary_op(builder, |builder, a, b| {
                        debug!("mod {a:?} % {b:?}");
                        let a_i64 = builder.ins().fcvt_to_sint(types::I64, a);
                        let b_i64 = builder.ins().fcvt_to_sint(types::I64, b);
                        let res_i64 = builder.ins().srem(a_i64, b_i64);
                        builder.ins().fcvt_from_sint(types::F64, res_i64)
                    })?;
                }
                // Calculates only integer powers
                // TODO: define a function for calculating powers
                Bytecode::Pow => {
                    self.binary_op(builder, |builder, a, b| {
                        let result = builder.ins().call(*pow_func_ref, &[a, b]);
                        builder.inst_results(result)[0]
                    })?;
                }
                Bytecode::Or => self.binary_op(builder, |builder, a, b| {
                    debug!("{a:?} OR {b:?}");
                    let zero_val = builder.ins().f64const(0.0);
                    let a_bool = builder.ins().fcmp(FloatCC::NotEqual, a, zero_val);
                    let b_bool = builder.ins().fcmp(FloatCC::NotEqual, b, zero_val);
                    let res_bool = builder.ins().bor(a_bool, b_bool);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::And => self.binary_op(builder, |builder, a, b| {
                    debug!("{a:?} AND {b:?}");
                    let zero_val = builder.ins().f64const(0.0);
                    let a_bool = builder.ins().fcmp(FloatCC::NotEqual, a, zero_val);
                    let b_bool = builder.ins().fcmp(FloatCC::NotEqual, b, zero_val);
                    let res_bool = builder.ins().band(a_bool, b_bool);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Not => {
                    let zero_val = builder.ins().f64const(0.0);
                    let a = self.pop_value()?;
                    debug!("NOT {a:?}");
                    let a_bool = builder.ins().fcmp(FloatCC::Equal, a, zero_val);
                    let res_bool = builder.ins().bnot(a_bool);
                    let res = builder.ins().fcvt_from_sint(types::F64, res_bool);
                    self.stack.push(res);
                }
                Bytecode::Eq => self.binary_op(builder, |builder, a, b| {
                    debug!("Eq {a:?} == {b:?}");
                    let res_bool = builder.ins().fcmp(FloatCC::Equal, a, b);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Ne => self.binary_op(builder, |builder, a, b| {
                    debug!("Ne {a:?} != {b:?}");
                    let res_bool = builder.ins().fcmp(FloatCC::NotEqual, a, b);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Gt => self.binary_op(builder, |builder, a, b| {
                    debug!("Gt {a:?} > {b:?}");
                    let res_bool = builder.ins().fcmp(FloatCC::GreaterThan, a, b);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Ge => self.binary_op(builder, |builder, a, b| {
                    debug!("Ge {a:?} >= {b:?}");
                    let res_bool = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, a, b);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Lt => self.binary_op(builder, |builder, a, b| {
                    debug!("Lt {a:?} < {b:?}");
                    let res_bool = builder.ins().fcmp(FloatCC::LessThan, a, b);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Le => self.binary_op(builder, |builder, a, b| {
                    debug!("Le {a:?} <= {b:?}");
                    let res_bool = builder.ins().fcmp(FloatCC::LessThanOrEqual, a, b);
                    builder.ins().fcvt_from_sint(types::F64, res_bool)
                })?,
                Bytecode::Call(func_name, arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.stack.pop().ok_or("Stack underflow")?);
                    }
                    args.reverse();

                    let func_ref = func_refs
                        .get(func_name)
                        .ok_or(format!("Undefined function: {func_name}"))?;

                    trace!("func_refs: {func_refs:#?}");
                    debug!("arg_count: {arg_count:?}");
                    debug!("func_ref: {func_ref:?}");
                    debug!("func args: {args:?}");
                    let call = builder.ins().call(*func_ref, &args);
                    let res = builder.inst_results(call)[0];
                    self.stack.push(res);
                }
                Bytecode::LoadVariable(name) => {
                    // TODO: check if we need to push into a vec
                    // Just using the total no. of variables might suffice
                    variables.push(name.to_string());
                    let offset = (index * 8) as i32; // f64 values -> 8 bytes each
                    let var_value =
                        builder
                            .ins()
                            .load(types::F64, MemFlags::new(), memory_ptr, offset);

                    index += 1;
                    self.stack.push(var_value);
                }
                // Bytecode::LoadVariable(name) => {
                //     let global_value = var_refs
                //         .get(name)
                //         .ok_or(format!("Undefined variable: {name}"))?;
                //     trace!("var_refs: {var_refs:#?}");
                //     debug!("var_ref: {global_value:#?}");
                //
                //     builder.create_global_value(GlobalValueData::Symbol {
                //         name: ExternalName::User(UserExternalNameRef::new(0)),
                //         offset: Imm64::new(0),
                //         tls: false,
                //         colocated: false,
                //     });
                //     // GlobalValueData
                //     let global_value_address =
                //         builder.ins().global_value(types::I64, *global_value);
                //     let res =
                //         builder
                //             .ins()
                //             .load(types::F64, MemFlags::new(), global_value_address, 0);
                //     self.stack.push(res);
                // }
                // Bytecode::StoreVariable(name) => {
                //     let value = self.stack.pop().ok_or("Stack underflow")?;
                //     variables.insert(name.clone(), value);
                // }
                // Bytecode::GetProperty(prop) => {
                //     let value = self.pop_value()?;
                //
                //     // // If the value is a struct, use static offsets
                //     // if let Some(offset) = self.struct_offsets.get(prop) {
                //     //     let addr = builder.ins().load(mem::ptr_ty(), value, *offset);
                //     //     self.push_value(addr);
                //     // }
                //     // // Otherwise, fall back to hashmap lookup
                //     // else {
                //
                //     let key = self.const_string(prop);
                //     let result = self.call_builtin_function("hashmap_lookup", &[value?, key]);
                //     self.push_value(result?);
                //     // }
                // }
                // Bytecode::Jump(target) => {
                //     let target_block = *block_map.get(target).unwrap();
                //     builder.ins().jump(target_block, &[]);
                // }
                // Bytecode::JumpIfTrue(target) => {
                //     let target_block = *block_map.get(target).unwrap();
                //     let cond = self.pop_value();
                //     builder.ins().brnz(cond, target_block, &[]);
                // }
                // Bytecode::JumpIfFalse(target) => {
                //     let target_block = *block_map.get(target).unwrap();
                //     let cond = self.pop_bool();
                //     builder.ins().brz(cond, target_block, &[]);
                // }
                // Bytecode::Return => {
                //     if let Some(value) = self.stack.pop() {
                //         builder.ins().return_(&[value]);
                //     } else {
                //         return Err("Return with empty stack".into());
                //     }
                // }
                Bytecode::NoOp => {}
                _ => return Err("invalid bytecode".to_string()),
            }
        }

        Ok(variables)
    }

    /// **Executes compiled JIT function**
    pub fn execute(&mut self, func_id: *const u8, memory_ptr: *mut f64) -> Result<f64, String> {
        debug!("memory_ptr: {:?}", memory_ptr.is_null());

        if !memory_ptr.is_null() {
            let func: extern "C" fn(*mut f64) -> f64 = unsafe { std::mem::transmute(func_id) };
            Ok(func(memory_ptr))
        } else {
            let func: extern "C" fn() -> f64 = unsafe { std::mem::transmute(func_id) };
            Ok(func())
        }
    }

    /// Extracts a value from the stack
    fn pop_value(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())
    }
}
