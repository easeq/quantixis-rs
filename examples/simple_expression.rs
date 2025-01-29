// use quantixis_rs::ast::{Compiler, Evaluator, Executor};
use quantixis_rs::ast::{Compiler, Executor, Parser, Value};
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

    let expression = "price > 100 AND NOT volume < 5000 OR volume >= 3000";
    let mut executor = Executor::new();
    let result = executor.execute_expression(expression, &context);
    println!("Result: {:?}", result);

    // match evaluator.evaluate_expression(expression, &context) {
    //     Ok(result) => println!("Result: {}", result),
    //     Err(err) => println!("Error: {}", err),
    // }
}
