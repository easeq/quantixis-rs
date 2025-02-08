// use quantixis_rs::ast::{Compiler, Executor};
// use quantixis_rs::ast::{Compiler, Executor, Parser};
use cranelift::prelude::{types, AbiParam};
use log::{debug, trace};
use quantixis_macros::quantinxis_fn;
// use quantixis_rs::bytecode::Bytecode;
use quantixis_rs::bytecode::{BytecodeCompiler, Value};
use quantixis_rs::jit::{execute, ArrayMeta, JITCompilerBuilder};
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

fn _sum_array(a: f64) -> f64 {
    let mut sum: f64 = 0.0;
    // println!("a: {:?}", a as i64);
    // Convert the stored i64 back into a pointer to ArrayMeta.
    let meta_ptr = a as i64 as *mut ArrayMeta;
    if meta_ptr.is_null() {
        println!("Array meta pointer is null");
    } else {
        // Safety: We assume the pointer is valid and was allocated via set_array_i32.
        let meta = unsafe { &*meta_ptr };
        // Recover the original array pointer by casting meta.ptr to a pointer of the correct type.
        let array_ptr = meta.ptr as *const i32;
        // Construct a slice from the raw pointer and the length stored in meta.
        let arr_slice = unsafe { std::slice::from_raw_parts(array_ptr, meta.len) };
        println!("Reconstructed array: {:?}", arr_slice);
        // println!("v: {:?}", v);
        sum = arr_slice
            .to_vec()
            .into_iter()
            .reduce(|acc, e| acc + e)
            .unwrap() as f64;
    }

    sum as f64
}

fn main() {
    pretty_env_logger::init();

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
        .add_function(
            "sum_array".to_string(),
            _sum_array as *const u8,
            vec![AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        );

    let mut jit = builder.build().expect("Failed to build JIT compiler");

    let exprs = [
        "10 + 5",
        "10 + 5 * 2",
        "10 == true",
        "10 != true",
        "10 > true",
        "10 >= true",
        "10 < true",
        "10 <= true",
        "2 ^ 32",
        "2 ^ 2 * 3",
        "true > 10",
        "(true > 10) + 3",
        "(true > 10) + 3 OR 5 > 3",
        "a",
        "a + b",
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
        // "a ^ b",
        // "a ^ b * 10",
        // "b ^ a",
        // "(a ^ b) > (b ^ a)",
        "add(a, b)",
        "multiply(a, b)",
        "add(a, b) > multiply(a, b)",
        "square(a)",
        "multiply(5, 11)",
        "square(6)",
        // "sum_array(a)",
    ];
    for expr in exprs {
        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        trace!("bytecode: {bytecode:#?}");

        let (func_id, mut env) = jit.compile(&bytecode).unwrap();

        let a = rand::random_range(0.0..20.0);
        let b = rand::random_range(0.0..20.0);

        // let v: Box<[i32]> = vec![1, 2, 3, 4, 5].into_boxed_slice();
        // env.set_array_i32("a", v);

        // println!(
        //     "ptr: {:?}, ptr_i64: {:?}, capacity: {}, len: {}",
        //     v.as_mut_ptr(),
        //     v.as_mut_ptr() as i64,
        //     v.capacity(),
        //     v.len()
        // );
        env.set_f64("a", a);
        env.set_f64("b", b);

        env.init();

        let vars_ptr = env.as_ptr();

        debug!("vars_ptr: {vars_ptr:?} {:?}", vars_ptr.is_null());

        let result = execute(func_id, vars_ptr).unwrap();
        println!("{expr:?} = {}, (vars: {:?})", result, [a, b]);
    }
}
