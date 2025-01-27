use quantixis_rs::ast::Evaluator;
use std::collections::HashMap;

fn main() {
    pretty_env_logger::init();

    let mut evaluator = Evaluator::new(100);

    let context: HashMap<String, f64> =
        [("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]
            .iter()
            .cloned()
            .collect();

    // let expression = "price > 100 AND volume < 5000 AND volume >= 3001";
    // let expression = "(price > 100 AND volume < 5000) OR volume >= 3000";
    // let expression = "price > 100 AND volume < 5000 OR volume > 3000";

    let expression = "indicator.result.signal";
    evaluator.parse_expression(&expression).unwrap();

    match evaluator.evaluate_expression(expression, &context) {
        Ok(result) => println!("Result: {}", result),
        Err(err) => println!("Error: {}", err),
    }
}
