use quantixis_rs::ast::Evaluator;
use std::collections::HashMap;

fn main() {
    pretty_env_logger::init();

    let mut evaluator = Evaluator::new(100);

    let expression = "price > 50 AND volume < 5000";
    let ast = evaluator
        .parse_expression(expression)
        .expect("Failed to parse");

    let context: HashMap<String, f64> =
        [("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]
            .iter()
            .cloned()
            .collect();

    match evaluator.evaluate_ast(&ast, &context) {
        Ok(result) => println!("Result: {}", result),
        Err(err) => println!("Error: {}", err),
    }
}
