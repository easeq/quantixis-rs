use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::prelude::*;
use cranelift_jit::JITModule;
use cranelift_module::{FuncId, Module};
use log::debug;

/// **Compiles bytecode to Cranelift IR**
pub fn pow(module: &mut JITModule) -> Result<FuncId, String> {
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
