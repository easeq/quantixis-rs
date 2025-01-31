use crate::ast::{Executor, Value};
use quantixis_macros::quantinxis_fn;

pub fn register(executor: &mut Executor) {
    executor.register_function("pivot_points", pivot_points);
}

#[quantinxis_fn]
fn pivot_points(high: f64, low: f64, close: f64) -> Result<Value, String> {
    let pivot = (high + low + close) / 3.0;
    Ok(Value::Number(pivot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_points() {
        let args = vec![
            Value::Number(110.0),
            Value::Number(100.0),
            Value::Number(105.0),
        ];
        let result = pivot_points(&args).unwrap();
        assert!(matches!(result, Value::Number(105.0)));
    }
}
