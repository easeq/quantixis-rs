pub mod momentum;
pub mod other;
pub mod trend;
pub mod volatility;
pub mod volume;

use crate::ast::Executor;

pub fn register_functions(executor: &mut Executor) {
    momentum::register(executor);
    trend::register(executor);
    volatility::register(executor);
    volume::register(executor);
}
