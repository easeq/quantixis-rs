use crate::ast::{FunctionArgs, FunctionResult};
use crate::Evaluator;

pub fn register(evaluator: &mut Evaluator) {
    evaluator.register_function("on_balance_volume", on_balance_volume);
    evaluator.register_function("chaikin_money_flow", chaikin_money_flow);
}

pub fn on_balance_volume(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLCV data
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period * 4 {
        return Err("Insufficient data for OBV calculation".to_string());
    }

    let mut obv = 0.0;

    for i in 1..values.len() {
        let close_today = values[i * 4 + 2];
        let close_previous = values[(i - 1) * 4 + 2];
        let volume_today = values[i * 4 + 3];

        if close_today > close_previous {
            obv += volume_today;
        } else if close_today < close_previous {
            obv -= volume_today;
        }
    }

    Ok(FunctionResult::UnnamedF64(obv))
}

pub fn chaikin_money_flow(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLCV data
    let period = args.get_number("period").unwrap_or(20.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for CMF calculation".to_string());
    }

    let mut money_flow_volume: f64 = 0.0;
    let mut volume: f64 = 0.0;

    for i in 0..period {
        let high = values[i * 4];
        let low = values[i * 4 + 1];
        let close = values[i * 4 + 2];
        let volume_at_period = values[i * 4 + 3];

        let mfv = ((close - low) - (high - close)) / (high - low) * volume_at_period;
        money_flow_volume += mfv;
        volume += volume_at_period;
    }

    let cmf = money_flow_volume / volume;
    Ok(FunctionResult::UnnamedF64(cmf))
}
