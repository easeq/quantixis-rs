use quantixis_rs::ast::{Compiler, Executor};
// use quantixis_rs::ast::{Compiler, Executor, Parser};
use log::debug;
use quantixis_macros::quantinxis_fn;
use quantixis_rs::bytecode::{BytecodeCompiler, BytecodeExecutor, Value};
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

fn main() {
    pretty_env_logger::init();

    let mut executor = Executor::new();

    let expr = "2";
    let compiled = Compiler::compile_expression(expr).unwrap();
    debug!("compiled: {compiled:?}");

    let mut bytecode_exec = BytecodeExecutor::new();
    let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
    debug!("bytecode: {bytecode:?}");

    bytecode_exec.execute(&bytecode).unwrap();

    let _ = executor.execute(&compiled, &Default::default());

    // // let mut evaluator = Evaluator::new(100);
    //
    // let context: HashMap<String, Value> = HashMap::from([
    //     ("price".to_string(), Value::Number(120.0)),
    //     ("volume".to_string(), Value::Number(3000.0)),
    // ]);
    //
    // // let expression = "price > 100 AND volume < 5000 AND volume >= 3001";
    // // let expression = "(price > 100 AND volume < 5000) OR volume >= 3000";
    // // let expression = "price > 100 AND volume < 5000 OR volume > 3000";
    //
    // // let expression = "price > 100 AND NOT volume < 5000 OR volume >= 3000";
    // // let mut executor = Executor::new();
    // // let result = executor.execute_expression(expression, &context);
    // // println!("Result: {:?}", result);
    //
    // // let expr = "NOT true";
    // // let mut compiler = BytecodeCompiler::new();
    // // let bytecode = compiler.compile(expr).unwrap();
    // // let result = BytecodeExecutor::new().execute(&bytecode).unwrap().unwrap();
    // // println!("result: {:#?}", result);
    //
    // let expr = "add(1, multiply(2, subtract(5, divide(10, 2))))";
    //
    // let mut executor = BytecodeExecutor::new();
    // executor.bind_variable(
    //     "user",
    //     Value::Map(HashMap::from([(
    //         "profile".to_string(),
    //         Value::Map(HashMap::from([("score".to_string(), Value::Number(2.0))])),
    //     )])),
    // );
    // executor.bind_variable(
    //     "obj",
    //     Value::Map(HashMap::from([("flag".to_string(), Value::Boolean(true))])),
    // );
    // let mut compiler = BytecodeCompiler::new();
    // compiler.register_function("add", add);
    // compiler.register_function("divide", divide);
    // compiler.register_function("subtract", subtract);
    // compiler.register_function("multiply", multiply);
    // compiler.register_function("square", square);
    //
    // let bytecode = compiler.compile(expr).unwrap();
    // let result = executor.execute(&bytecode).unwrap();
    // println!("result: {result:#?}");
    //
    // // match evaluator.evaluate_expression(expression, &context) {
    // //     Ok(result) => println!("Result: {}", result),
    // //     Err(err) => println!("Error: {}", err),
    // // }
}
