use crate::ast::{Executor, Value};

pub fn register(executor: &mut Executor) {
    executor.register_function("pivot_points", pivot_points);
}

fn pivot_points(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "Pivot Points require 3 arguments: (high: f64, low: f64, close: f64)".to_string(),
        );
    }

    let high = match args[0] {
        Value::Number(h) => h,
        _ => return Err("First argument must be a number representing the high price".to_string()),
    };

    let low = match args[1] {
        Value::Number(l) => l,
        _ => return Err("Second argument must be a number representing the low price".to_string()),
    };

    let close = match args[2] {
        Value::Number(c) => c,
        _ => {
            return Err("Third argument must be a number representing the close price".to_string())
        }
    };

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
