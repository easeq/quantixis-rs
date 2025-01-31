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

#[macro_export]
macro_rules! extract_args {
    ($args:expr, $($name:ident: $typ:tt),+) => {{
        use crate::ast::Value;
        let mut iter = $args.iter();
        let extracted = (
            $(match iter.next() {
                Some(Value::$typ(value)) => value,
                _ => return Err(format!(
                    "Expected argument '{}' to be of type '{}'",
                    stringify!($name),
                    stringify!($typ)
                )),
            }),+
        );
        extracted
    }};
}

#[macro_export]
macro_rules! extract_args_bytecode {
    ($args:expr, $($name:ident: $typ:ident),+) => {{
        let mut iter = $args.iter();
        let extracted = (
            $(
                match iter.next() {
                    Some($crate::bytecode::Value::$typ(value)) => value,
                    _ => panic!(
                        "Expected argument '{}' to be of type '{}'",
                        stringify!($name),
                        stringify!($typ)
                    ),
                }
            ),+
        );
        extracted
    }};
}
