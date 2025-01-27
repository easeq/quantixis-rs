pub mod ast;
pub mod functions;

use ast::{Evaluator, Parser};
use functions::register_functions;

pub fn evaluate_expression(
    expression: &str,
    context: &std::collections::HashMap<String, f64>,
) -> Result<f64, String> {
    let ast = Parser::parse_expression(expression)?;

    let mut evaluator = Evaluator::new(100);
    register_functions(&mut evaluator);
    evaluator.evaluate(&ast, context)
}
