pub mod momentum;
pub mod other;
pub mod trend;
pub mod volatility;
pub mod volume;

use crate::ast::Evaluator;

pub fn register_functions(evaluator: &mut Evaluator) {
    momentum::register(evaluator);
    trend::register(evaluator);
    volatility::register(evaluator);
    volume::register(evaluator);
}
