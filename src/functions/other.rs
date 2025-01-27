use crate::ast::{FunctionArgs, FunctionResult};
use crate::Evaluator;
use std::collections::HashMap;

pub fn register(evaluator: &mut Evaluator) {
    evaluator.register_function("pivot_points", pivot_points);
}

pub fn pivot_points(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    if values.len() < 3 {
        return Err("Insufficient data for Pivot Point calculation".to_string());
    }

    let high = values[0];
    let low = values[1];
    let close = values[2];

    let pivot = (high + low + close) / 3.0;
    let support1 = (2.0 * pivot) - high;
    let resistance1 = (2.0 * pivot) - low;
    let support2 = pivot - (high - low);
    let resistance2 = pivot + (high - low);

    Ok(FunctionResult::NamedF64Map(HashMap::from([
        ("support1".to_string(), support1),
        ("resistance1".to_string(), resistance1),
        ("support2".to_string(), support2),
        ("resistance2".to_string(), resistance2),
    ])))
}
