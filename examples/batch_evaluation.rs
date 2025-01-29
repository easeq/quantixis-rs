use quantixis_rs::ast::{Executor, Value};
use std::collections::HashMap;

fn main() {
    pretty_env_logger::init();

    let contexts = vec![
        HashMap::from([
            ("price".to_string(), Value::Number(120.0)),
            ("volume".to_string(), Value::Number(3000.0)),
        ]),
        HashMap::from([
            ("price".to_string(), Value::Number(80.0)),
            ("volume".to_string(), Value::Number(6000.0)),
        ]),
    ];

    let expression = "price > 100 AND volume < 5000";

    let mut executor = Executor::new();
    for (i, context) in contexts.iter().enumerate() {
        let result = executor.execute_expression(expression, &context).unwrap();
        println!("Result {}: {:?}", i, result);
    }
}
