use crate::bytecode::{Bytecode, Value as BytecodeValue};
use cranelift::codegen::ir::{FuncRef, Function, GlobalValue, UserExternalNameRef};
use cranelift::codegen::isa::CallConv;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataId, FuncId, Linkage, Module};
use log::{debug, trace};
use std::collections::HashMap;

/// **Compiles bytecode to Cranelift IR**
pub fn compile_pow_fn(module: &mut JITModule) -> Result<FuncId, String> {
    let mut ctx = module.make_context();
    ctx.func.signature.params.push(AbiParam::new(types::F64));
    ctx.func.signature.params.push(AbiParam::new(types::F64));
    ctx.func.signature.returns.push(AbiParam::new(types::F64));

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

    let main_block = builder.create_block();
    builder.switch_to_block(main_block);
    builder.append_block_params_for_function_params(main_block);
    // main block
    // let result = builder.ins().f64const(29.0);
    builder.seal_block(main_block);

    let a = builder.block_params(main_block)[0];
    let b = builder.block_params(main_block)[1];

    let b_i64 = builder.ins().fcvt_to_sint(types::I64, b);

    debug!("pow {a:?} ^ {b_i64:?}");
    // Create loop_block with 2 params (result: f64, b: i64)
    let loop_block = builder.create_block();
    builder.append_block_param(loop_block, types::F64);
    builder.append_block_param(loop_block, types::I64);

    // Create loop_exit with 1 param (result: f64)
    let loop_exit = builder.create_block();
    builder.append_block_param(loop_exit, types::F64);

    // Initialize result = 1.0
    let result = builder.ins().f64const(1.0);
    // If b == 0, branch to loop_exit block (a^0 = 1)
    builder
        .ins()
        .brif(b_i64, loop_block, &[result, b_i64], loop_exit, &[result]);

    // loop_block:
    {
        builder.switch_to_block(loop_block);
        let mut result = builder.block_params(loop_block)[0];
        let mut b_i64 = builder.block_params(loop_block)[1];
        let b_dec = builder.ins().iconst(types::I64, 1);
        // result = result * a
        result = builder.ins().fmul(result, a);
        // b_i64 = b_i64 - b_dec
        b_i64 = builder.ins().isub(b_i64, b_dec);

        // If b != 0, continue loop, otherwise exit
        builder
            .ins()
            .brif(b_i64, loop_block, &[result, b_i64], loop_exit, &[result]);
        builder.seal_block(loop_block);
    }

    // loop_exit:
    {
        builder.switch_to_block(loop_exit);
        builder.seal_block(loop_exit);
        let result = builder.block_params(loop_exit)[0];
        builder.ins().return_(&[result]);
    }

    builder.finalize();

    let func_id = module
        .declare_anonymous_function(&ctx.func.signature)
        .unwrap();
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| e.to_string())?;
    module.finalize_definitions().map_err(|e| e.to_string())?;

    Ok(func_id)
}

pub struct JITCompiler {
    module: JITModule,
    functions_map: HashMap<String, FuncId>,
    data_map: HashMap<String, DataId>,
    stack: Vec<Value>,
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
    pub fn compile(&mut self, bytecode: &[Bytecode]) -> Result<*const u8, String> {
        let pow_func_id = compile_pow_fn(&mut self.module)?;

        let mut ctx = self.module.make_context();
        ctx.func.signature.returns.push(AbiParam::new(types::F64));

        let func_refs = self.link_external_functions(&mut ctx)?;
        let var_refs = self.link_external_vars(&mut ctx)?;

        let pow_func_ref = self.module.declare_func_in_func(pow_func_id, &mut ctx.func);

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let main_block = builder.create_block();
        builder.switch_to_block(main_block);
        builder.append_block_params_for_function_params(main_block);
        // main block
        {
            let _ = self.compile_main_block(
                &mut builder,
                &var_refs,
                &func_refs,
                bytecode,
                &pow_func_ref,
            )?;
            builder.seal_block(main_block);
        }

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

        let func = self.module.get_finalized_function(func_id);
        trace!("finalized function: {func:#?}");
        Ok(func as *const u8)
    }

    fn compile_main_block(
        &mut self,
        builder: &mut FunctionBuilder,
        var_refs: &HashMap<String, GlobalValue>,
        func_refs: &HashMap<String, FuncRef>,
        bytecode: &[Bytecode],
        pow_func_ref: &FuncRef,
    ) -> Result<(), String> {
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
                    let global_value = var_refs
                        .get(name)
                        .ok_or(format!("Undefined variable: {name}"))?;
                    trace!("var_refs: {var_refs:#?}");
                    debug!("var_ref: {global_value:#?}");

                    builder.create_global_value(GlobalValueData::Symbol {
                        name: ExternalName::User(UserExternalNameRef::new(0)),
                        offset: Imm64::new(0),
                        tls: false,
                        colocated: false,
                    });
                    // GlobalValueData
                    let global_value_address =
                        builder.ins().global_value(types::I64, *global_value);
                    let res =
                        builder
                            .ins()
                            .load(types::F64, MemFlags::new(), global_value_address, 0);
                    self.stack.push(res);
                }
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

        Ok(())
    }

    /// **Executes compiled JIT function**
    pub fn execute(
        &mut self,
        func_id: *const u8,
        vars: HashMap<String, *const u8>,
    ) -> Result<f64, String> {
        // let code_ptr = self.module.get_finalized_function(func_id);
        let func = unsafe { std::mem::transmute::<_, fn() -> f64>(func_id) };
        Ok(func())
    }

    /// Extracts a value from the stack
    fn pop_value(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())
    }
}

// pub fn build_pow_func(&mut self) -> Result<(*const u8, FuncRef), String> {
//     let mut ctx = self.module.make_context();
//     let mut context = FunctionBuilderContext::new();
//     let mut function = Function::new();
//
//     let mut sig = Signature::new(CallConv::SystemV);
//     sig.params.push(AbiParam::new(types::F64)); // a: float
//     sig.params.push(AbiParam::new(types::I64)); // b: int
//     sig.returns.push(AbiParam::new(types::F64)); // return: float
//
//     function.signature = sig;
//
//     let mut builder = FunctionBuilder::new(&mut function, &mut context);
//     let main_block = builder.create_block();
//     builder.switch_to_block(main_block);
//     builder.append_block_params_for_function_params(main_block);
//     // main block
//     let f = builder.ins().f64const(1.0);
//     builder.ins().return_(&[f]);
//     builder.seal_block(main_block);
//
//     builder.finalize();
//
//     let func_id = self
//         .module
//         .declare_anonymous_function(&ctx.func.signature)
//         .map_err(|e| e.to_string())?;
//
//     debug!("3");
//     use cranelift::codegen::settings::Flags;
//     // self.module
//     //     .define_function(func_id, &mut ctx)
//     //     .map_err(|e| e.to_string())?;
//     let flag_builder = cranelift::codegen::settings::builder();
//     let flags = Flags::new(flag_builder);
//     let isa = isa::lookup_by_name("x86_64")
//         .unwrap()
//         .finish(flags)
//         .unwrap();
//     // Create the DominatorTree (required for compiling)
//     let mut domtree = codegen::dominator_tree::DominatorTree::new();
//
//     // Create a ControlPlane (required for compilation)
//     let mut ctrl_plane = codegen::control::ControlPlane::default();
//     // Compile the functions
//     let mut compiled_power = isa
//         .compile_function(
//             &function,
//             &domtree,
//             false, // want_disasm (we don't need disassembly here)
//             &mut ctrl_plane,
//         )
//         .unwrap();
//     debug!("4");
//     self.module
//         .finalize_definitions()
//         .map_err(|e| e.to_string())?;
//
//     debug!("5");
//     let func = self.module.get_finalized_function(func_id);
//     let func_ref = self.module.declare_func_in_func(func_id, &mut ctx.func);
//
//     trace!("finalized power function: {func:#?}");
//     Ok((func as *const u8, func_ref))
// }

// pub fn compile_pow_fn(&mut self) -> Result<*const u8, String> {
//     // Cranelift function to compute `float ^ int`
//     let mut context = codegen::Context::new();
//
//     // Create Cranelift function signature
//     let mut sig = Signature::new(CallConv::SystemV);
//     sig.params.push(AbiParam::new(types::F64)); // a: float
//     sig.params.push(AbiParam::new(types::I64)); // b: int
//     sig.returns.push(AbiParam::new(types::F64)); // return: float
//
//     let mut func = Function::with_name_signature(
//         codegen::ir::UserFuncName::User(codegen::ir::UserExternalName::new(0, 1)),
//         sig.clone(),
//     );
//
//     // Create FunctionBuilder and its context
//     let mut builder_ctx = FunctionBuilderContext::new();
//     let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);
//
//     // Create the entry block
//     let entry = builder.create_block();
//     builder.append_block_params_for_function_params(entry);
//     builder.switch_to_block(entry);
//
//     // Load the parameters from the function
//     let a = builder.block_params(entry)[0];
//     let b = builder.block_params(entry)[1];
//
//     // Convert b (i64) to a signed integer
//     let b_i64 = builder.ins().fcvt_to_sint(types::I64, b);
//
//     // Debugging print (optional)
//     // debug!("pow {a:?} ^ {b:?}");
//
//     // Create loop_block with 2 params (result: f64, b: i64)
//     let loop_block = builder.create_block();
//     builder.append_block_param(loop_block, types::F64);
//     builder.append_block_param(loop_block, types::I64);
//
//     // Create loop_exit with 1 param (result: f64)
//     let loop_exit = builder.create_block();
//     builder.append_block_param(loop_exit, types::F64);
//
//     // Initialize result = 1.0
//     let result = builder.ins().f64const(1.0);
//
//     // If b == 0, branch to loop_exit block (a^0 = 1)
//     builder
//         .ins()
//         .brif(b_i64, loop_block, &[result, b_i64], loop_exit, &[result]);
//
//     // loop_block:
//     {
//         builder.switch_to_block(loop_block);
//         let mut result = builder.block_params(loop_block)[0];
//         let mut b_i64 = builder.block_params(loop_block)[1];
//         let b_dec = builder.ins().iconst(types::I64, 1);
//
//         // result = result * a
//         result = builder.ins().fmul(result, a);
//
//         // b_i64 = b_i64 - b_dec
//         b_i64 = builder.ins().isub(b_i64, b_dec);
//
//         // If b != 0, continue loop, otherwise exit
//         builder
//             .ins()
//             .brif(b_i64, loop_block, &[result, b_i64], loop_exit, &[result]);
//         builder.seal_block(loop_block);
//     }
//
//     // loop_exit:
//     {
//         builder.switch_to_block(loop_exit);
//         let res = builder.block_params(loop_exit)[0];
//         builder.seal_block(loop_exit);
//
//         // Push the result onto the stack
//         builder.ins().return_(&[res]);
//     }
//
//     // Finish building the function
//     builder.finalize();
//
//     // // Set up the Cranelift backend
//     // let mut isa_builder = X64::new(Flags::new(cranelift::codegen::settings::builder()));
//     // let isa = isa_builder.finish();
//     // let mut compiled = isa.compile(&func).unwrap();
//     //
//     // // Optionally, print the compiled function (for debugging)
//     // println!("{:?}", compiled);
//     //
//     // // Now we can execute the function.
//     // let exec_func = unsafe {
//     //     let exec_func = compiled.as_ptr() as *mut unsafe extern "C" fn(f64, i64) -> f64;
//     //     (*exec_func)(2.0, 3) // Example: calculate 2^3
//     // };
// }
