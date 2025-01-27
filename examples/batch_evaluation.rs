use quantixis_rs::ast::Evaluator;
use std::collections::HashMap;

fn main() {
    pretty_env_logger::init();

    let mut evaluator = Evaluator::new(100);

    let contexts = vec![
        [("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]
            .iter()
            .cloned()
            .collect::<HashMap<String, f64>>(),
        [("price".to_string(), 80.0), ("volume".to_string(), 6000.0)]
            .iter()
            .cloned()
            .collect::<HashMap<String, f64>>(),
    ];

    let expression = "price > 100 AND volume < 5000";

    for (i, context) in contexts.iter().enumerate() {
        match evaluator.evaluate_expression(expression, context) {
            Ok(result) => println!("Batch {}: Result = {}", i + 1, result),
            Err(err) => println!("Batch {}: Error = {}", i + 1, err),
        }
    }
}
