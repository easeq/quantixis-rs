use quantixis_rs::ast::{Executor, Value};
use std::collections::HashMap;

fn main() {
    pretty_env_logger::init();

    let mut executor = Executor::new();

    let expression = "price > 50 AND volume < 5000";
    let ast = executor
        .parse_expression(expression)
        .expect("Failed to parse");

    let context: HashMap<String, Value> = HashMap::from([
        ("price".to_string(), Value::Number(120.0)),
        ("volume".to_string(), Value::Number(3000.0)),
    ]);

    match executor.execute_ast(&ast, &context) {
        Ok(result) => println!("Result: {:?}", result),
        Err(err) => println!("Error: {}", err),
    }
}
