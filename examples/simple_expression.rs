// use quantixis_rs::ast::{Compiler, Evaluator, Executor};
use quantixis_rs::ast::{Compiler, Executor, Parser};
use quantixis_rs::bytecode::{BytecodeCompiler, BytecodeExecutor, Value};
use quantixis_rs::extract_args_bytecode;
use std::collections::HashMap;

fn main() {
    pretty_env_logger::init();

    // let mut evaluator = Evaluator::new(100);

    let context: HashMap<String, Value> = HashMap::from([
        ("price".to_string(), Value::Number(120.0)),
        ("volume".to_string(), Value::Number(3000.0)),
    ]);

    // let expression = "price > 100 AND volume < 5000 AND volume >= 3001";
    // let expression = "(price > 100 AND volume < 5000) OR volume >= 3000";
    // let expression = "price > 100 AND volume < 5000 OR volume > 3000";

    // let expression = "price > 100 AND NOT volume < 5000 OR volume >= 3000";
    // let mut executor = Executor::new();
    // let result = executor.execute_expression(expression, &context);
    // println!("Result: {:?}", result);

    // let expr = "NOT true";
    // let mut compiler = BytecodeCompiler::new();
    // let bytecode = compiler.compile(expr).unwrap();
    // let result = BytecodeExecutor::new().execute(&bytecode).unwrap().unwrap();
    // println!("result: {:#?}", result);

    let expr = "x + 2";

    let mut executor = BytecodeExecutor::new();
    executor.bind_variable(
        "user",
        Value::Map(HashMap::from([(
            "profile".to_string(),
            Value::Map(HashMap::from([("score".to_string(), Value::Number(2.0))])),
        )])),
    );
    executor.bind_variable(
        "obj",
        Value::Map(HashMap::from([("flag".to_string(), Value::Boolean(true))])),
    );
    // executor.register_function("add", |args| {
    //     use crate::extract_args_bytecode;
    //     let (a, b) = extract_args_bytecode!(args, a: Number, b: Number);
    //     Value::Number(*a + *b)
    // });
    // executor.register_function("divide", |args| {
    //     use crate::extract_args_bytecode;
    //     let (a, b) = extract_args_bytecode!(args, a: Number, b: Number);
    //     Value::Number(*a / *b)
    // });
    // executor.register_function("subtract", |args| {
    //     use crate::extract_args_bytecode;
    //     let (a, b) = extract_args_bytecode!(args, a: Number, b: Number);
    //     Value::Number(*a - *b)
    // });
    // executor.register_function("multiply", |args| {
    //     use crate::extract_args_bytecode;
    //     let (a, b) = extract_args_bytecode!(args, a: Number, b: Number);
    //     Value::Number(*a * *b)
    // });
    // executor.register_function("square", |args| {
    //     let a = extract_args_bytecode!(args, a: Number);
    //     Value::Number(*a * *a)
    // });

    let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
    let result = executor.execute(&bytecode).unwrap();
    println!("result: {result:#?}");

    // match evaluator.evaluate_expression(expression, &context) {
    //     Ok(result) => println!("Result: {}", result),
    //     Err(err) => println!("Error: {}", err),
    // }
}
