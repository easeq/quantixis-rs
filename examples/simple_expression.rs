// use quantixis_rs::ast::{Compiler, Executor};
// use quantixis_rs::ast::{Compiler, Executor, Parser};
use cranelift::prelude::{types, AbiParam};
use log::{debug, trace};
use quantixis_macros::quantinxis_fn;
// use quantixis_rs::bytecode::Bytecode;
use quantixis_rs::bytecode::{BytecodeCompiler, Value};
use quantixis_rs::jit::{execute, JITCompilerBuilder};
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

fn _sum_array(a: Vec<f64>) -> f64 {
    println!("{a:?}");
    let mut sum: f64 = 0.0;
    // unsafe {
    //     // Recover array.
    //     let arr_ptr = std::mem::transmute(a);
    //     // WARNING: Converting a fat pointer from raw parts requires care!
    //     // Here we simply print the pointer value.
    //     println!("  arr (raw pointer) = {:?}", arr_ptr);
    //     sum = arr_ptr.into_iter().reduce(|acc, e| acc + e).unwrap() as f64;
    //
    //     // // Recover hashmap.
    //     // let map_ptr = result.4 as *mut HashMap<String, i64>;
    //     // println!("  map (raw pointer) = {:?}", map_ptr);
    // }
    sum = a.into_iter().reduce(|acc, e| acc + e).unwrap();
    sum
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
        // "10 + 5",
        // "10 + 5 * 2",
        // "10 == true",
        // "10 != true",
        // "10 > true",
        // "10 >= true",
        // "10 < true",
        // "10 <= true",
        // "true > 10",
        // "(true > 10) + 3",
        // "(true > 10) + 3 OR 5 > 3",
        // "a",
        // "a + b",
        // "a + b * 2",
        // "a == b",
        // "a != b",
        // "a > b",
        // "a >= b",
        // "a < b",
        // "a <= b",
        // "a == true",
        // "a != true",
        // "a > true",
        // "a >= true",
        // "a < true",
        // "a <= true",
        // "a % b",
        // "a ^ b",
        // "a ^ b * 10",
        // "b ^ a",
        // "(a ^ b) > (b ^ a)",
        // "add(a, b)",
        // "multiply(a, b)",
        // "add(a, b) > multiply(a, b)",
        // "square(a)",
        // "multiply(5, 11)",
        // "square(6)",
        "sum_array(a)",
    ];
    for expr in exprs {
        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        trace!("bytecode: {bytecode:#?}");

        let (func_id, mut env) = jit.compile(&bytecode).unwrap();

        let a = rand::random_range(0.0..20.0);
        let b = rand::random_range(0.0..20.0);

        env.set_ptr(
            "a",
            Vec::from_iter(std::iter::repeat(5).take(10)).as_mut_ptr(),
        );
        env.set_f64("b", b);

        env.init();

        let vars_ptr = env.as_ptr();

        debug!("vars_ptr: {vars_ptr:?} {:?}", vars_ptr.is_null());

        let result = execute(func_id, vars_ptr).unwrap();
        println!("{expr:?} = {}, (vars: {:?})", result, [a, b]);
    }
}
