// use quantixis_rs::ast::{Compiler, Executor};
// use quantixis_rs::ast::{Compiler, Executor, Parser};
use cranelift::prelude::{types, AbiParam};
use log::debug;
use quantixis_macros::quantinxis_fn;
// use quantixis_rs::bytecode::Bytecode;
use quantixis_rs::bytecode::{BytecodeCompiler, Value};
use quantixis_rs::jit::JITCompilerBuilder;
use std::collections::HashMap;

#[quantinxis_fn]
fn add(a: f64, b: f64) -> Result<Value, String> {
    Ok(Value::Number(a + b))
}

#[quantinxis_fn]
fn subtract(a: f64, b: f64) -> Result<Value, String> {
    Ok(Value::Number(a - b))
}

#[quantinxis_fn]
fn multiply(a: f64, b: f64) -> Result<Value, String> {
    Ok(Value::Number(a * b))
}

#[quantinxis_fn]
fn multiply_result_obj(a: f64, b: f64) -> Result<Value, String> {
    Ok(Value::Map(HashMap::from([(
        "value".to_string(),
        Value::Number(a * b),
    )])))
}

#[quantinxis_fn]
fn divide(a: f64, b: f64) -> Result<Value, String> {
    Ok(Value::Number(a / b))
}

#[quantinxis_fn]
fn square(a: f64) -> Result<Value, String> {
    Ok(Value::Number(a * a))
}

// Sample Rust functions
fn _add(a: f64, b: f64) -> f64 {
    a + b
}

fn _multiply(a: f64, b: f64) -> f64 {
    a * b
}

fn _square(a: f64) -> f64 {
    a * a
}

// fn _pointer(a: *const u8) -> f64 {
//     // a * b
//     debug!("a: {a:?}");
//     0.0
// }

fn main() {
    pretty_env_logger::init();

    // let a = 4.0;
    // let a_ptr: *const u8 = &a as *const _ as *const u8;
    // debug!("a_addr: {a_ptr:?}");
    //
    // let b = 4.0;
    // let b_ptr: *const u8 = &b as *const _ as *const u8;
    // debug!("b_addr: {b_ptr:?}");

    let builder = JITCompilerBuilder::new()
        .add_function(
            "add".to_string(),
            _add as *const u8,
            vec![AbiParam::new(types::F64), AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        )
        .add_function(
            "multiply".to_string(),
            _multiply as *const u8,
            vec![AbiParam::new(types::F64), AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        )
        .add_function(
            "square".to_string(),
            _square as *const u8,
            vec![AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        )
        .add_variable("a".to_string(), &4.0 as *const _ as *const u8)
        .add_variable("b".to_string(), &10.0 as *const _ as *const u8);

    // let mut jit = JITCompiler::from(
    //     [
    //         ("add".to_string(), add as *const u8),
    //         ("multiply".to_string(), multiply as *const u8),
    //         // ("pointer".to_string(), _pointer as *const u8),
    //     ],
    //     [
    //         ("a".to_string(), &4.1 as *const _ as *const u8),
    //         ("b".to_string(), &10.0 as *const _ as *const u8),
    //     ],
    // );

    let mut jit = builder.build().expect("Failed to build JIT compiler");

    // let expr = "10 + 5 * 2";
    let exprs = [
        "10 + 5",
        "10 + 5 * 2",
        "10 == true",
        "10 != true",
        "10 > true",
        "10 >= true",
        "10 < true",
        "10 <= true",
        "true > 10",
        "(true > 10) + 3",
        "(true > 10) + 3 OR 5 > 3",
        "a + b * 2",
        "a == b",
        "a != b",
        "a > b",
        "a >= b",
        "a < b",
        "a <= b",
        "a == true",
        "a != true",
        "a > true",
        "a >= true",
        "a < true",
        "a <= true",
        "a % b",
        "a ^ b",
        "a ^ b * 10",
        "b ^ a",
        "(a ^ b) > (b ^ a)",
        "add(a, b)",
        "multiply(a, b)",
        "add(a, b) > multiply(a, b)",
        "square(a)",
        "multiply(5, 11)",
        "square(6)",
    ];
    for expr in exprs {
        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        debug!("bytecode: {bytecode:#?}");
        let (func_id, vars_ptr) = jit.compile(&bytecode).unwrap();
        debug!("vars_ptr: {vars_ptr:?} {:?}", vars_ptr.is_null());

        if !vars_ptr.is_null() {
            let a = rand::random_range(0.0..20.0) as i64 as f64;
            let a_ptr: *const u8 = &a as *const _ as *const u8;
            debug!("a_addr: {a_ptr:?}");

            let b = rand::random_range(0.0..20.0) as i64 as f64;
            let b_ptr: *const u8 = &b as *const _ as *const u8;
            debug!("b_addr: {b_ptr:?}");

            unsafe {
                *vars_ptr.offset(0) = a;
                *vars_ptr.offset(1) = b;
            }

            let func: extern "C" fn(*mut f64) -> f64 = unsafe { std::mem::transmute(func_id) };
            let result = func(vars_ptr);
            println!("{expr:?} = {}, (vars: {:?})", result, [a, b]);
        } else {
            let func: extern "C" fn() -> f64 = unsafe { std::mem::transmute(func_id) };
            let result = func();
            println!("{expr:?} = {}", result);
        }
    }
}
