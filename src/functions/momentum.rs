use crate::ast::{FunctionArgs, FunctionResult};
use crate::Evaluator;

pub fn register(evaluator: &mut Evaluator) {
    evaluator.register_function("rate_of_change", rate_of_change);
    evaluator.register_function("stochastic", stochastic);
    evaluator.register_function("momentum", momentum);
    evaluator.register_function("commodity_channel_index", commodity_channel_index);
    evaluator.register_function("chande_momentum_oscillator", chande_momentum_oscillator);
    evaluator.register_function("relative_vigor_index", relative_vigor_index);
    evaluator.register_function("williams_percent_r", williams_percent_r);
    evaluator.register_function("awesome_osc", awesome_oscillator);
    evaluator.register_function("ad_oscillator", ad_oscillator);
    evaluator.register_function("klinger_oscillator", klinger_oscillator);
    evaluator.register_function("choppiness_index", choppiness_index);
}

pub fn rate_of_change(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for ROC calculation".to_string());
    }

    let roc = (values[values.len() - 1] - values[values.len() - period])
        / values[values.len() - period]
        * 100.0;
    Ok(FunctionResult::UnnamedF64(roc))
}

pub fn stochastic(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLC data (high, low, close)
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period * 3 {
        return Err("Insufficient data for Stochastic calculation".to_string());
    }

    let high_prices: Vec<f64> = values.iter().step_by(3).take(period).cloned().collect();
    let low_prices: Vec<f64> = values
        .iter()
        .skip(1)
        .step_by(3)
        .take(period)
        .cloned()
        .collect();
    let close_prices: Vec<f64> = values
        .iter()
        .skip(2)
        .step_by(3)
        .take(period)
        .cloned()
        .collect();

    let highest_high = high_prices
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let lowest_low = low_prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let current_close = close_prices[close_prices.len() - 1];

    let stochastic_value = (current_close - lowest_low) / (highest_high - lowest_low) * 100.0;
    Ok(FunctionResult::UnnamedF64(stochastic_value))
}

pub fn momentum(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for Momentum calculation".to_string());
    }

    let momentum = values[values.len() - 1] - values[values.len() - period];
    Ok(FunctionResult::UnnamedF64(momentum))
}

pub fn commodity_channel_index(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for CCI calculation".to_string());
    }

    let mut typical_price_sum = 0.0;
    for i in 0..period {
        let high = values[i * 3];
        let low = values[i * 3 + 1];
        let close = values[i * 3 + 2];

        let typical_price = (high + low + close) / 3.0;
        typical_price_sum += typical_price;
    }

    let typical_price_avg = typical_price_sum / period as f64;
    let mut mean_deviation = 0.0;
    for i in 0..period {
        let high = values[i * 3];
        let low = values[i * 3 + 1];
        let close = values[i * 3 + 2];

        let typical_price = (high + low + close) / 3.0;
        mean_deviation += (typical_price - typical_price_avg).abs();
    }

    let mean_deviation_avg = mean_deviation / period as f64;
    let cci = (values[values.len() - 1] - typical_price_avg) / (0.015 * mean_deviation_avg);

    Ok(FunctionResult::UnnamedF64(cci))
}

pub fn chande_momentum_oscillator(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for CMO calculation".to_string());
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..period {
        let change = values[i] - values[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses -= change;
        }
    }

    let cmo = (gains - losses) / (gains + losses) * 100.0;
    Ok(FunctionResult::UnnamedF64(cmo))
}

pub fn relative_vigor_index(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLC data (open, high, low, close)
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period * 4 {
        return Err("Insufficient data for RVI calculation".to_string());
    }

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for i in 1..period {
        let open = values[i * 4];
        let close = values[i * 4 + 2];
        let high = values[i * 4 + 1];
        let low = values[i * 4 + 3];

        numerator += (close - open) + (high - low);
        denominator += (close - open) - (high - low);
    }

    let rvi = numerator / denominator;
    Ok(FunctionResult::UnnamedF64(rvi))
}

pub fn williams_percent_r(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLC data (high, low, close)
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period * 3 {
        return Err("Insufficient data for Williams %R calculation".to_string());
    }

    let high_prices: Vec<f64> = values.iter().step_by(3).take(period).cloned().collect();
    let low_prices: Vec<f64> = values
        .iter()
        .skip(1)
        .step_by(3)
        .take(period)
        .cloned()
        .collect();
    let close_prices: Vec<f64> = values
        .iter()
        .skip(2)
        .step_by(3)
        .take(period)
        .cloned()
        .collect();

    let highest_high = high_prices
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let lowest_low = low_prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let current_close = close_prices[close_prices.len() - 1];

    let williams_r = (highest_high - current_close) / (highest_high - lowest_low) * -100.0;
    Ok(FunctionResult::UnnamedF64(williams_r))
}

pub fn awesome_oscillator(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLC data (high, low)
    let short_period = args.get_number("short_period").unwrap_or(5.0) as usize;
    let long_period = args.get_number("long_period").unwrap_or(34.0) as usize;

    if values.len() < long_period * 2 {
        return Err("Insufficient data for Awesome Oscillator calculation".to_string());
    }

    let mut short_sma = 0.0;
    let mut long_sma = 0.0;

    for i in 0..short_period {
        short_sma += (values[i * 2] + values[i * 2 + 1]) / 2.0;
    }
    for i in 0..long_period {
        long_sma += (values[i * 2] + values[i * 2 + 1]) / 2.0;
    }

    let awesome_osc = short_sma / short_period as f64 - long_sma / long_period as f64;
    Ok(FunctionResult::UnnamedF64(awesome_osc))
}

pub fn ad_oscillator(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLCV data
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period * 4 {
        return Err("Insufficient data for AD Oscillator calculation".to_string());
    }

    let mut adl = 0.0;

    for i in 0..period {
        let high = values[i * 4];
        let low = values[i * 4 + 1];
        let close = values[i * 4 + 2];
        let volume = values[i * 4 + 3];

        let mfv = (close - low - (high - close)) / (high - low) * volume;
        adl += mfv;
    }

    Ok(FunctionResult::UnnamedF64(adl))
}

pub fn klinger_oscillator(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLCV data
    let fast_period = args.get_number("fast_period").unwrap_or(34.0) as usize;
    let slow_period = args.get_number("slow_period").unwrap_or(55.0) as usize;

    if values.len() < slow_period * 4 {
        return Err("Insufficient data for Klinger Oscillator calculation".to_string());
    }

    let mut money_flow_volumes = Vec::new();
    // let mut fast_ema_values = Vec::new();
    // let mut slow_ema_values = Vec::new();

    // Calculate the Money Flow Volume (MFV) for each bar
    for i in 1..values.len() {
        let high = values[i * 4];
        let low = values[i * 4 + 1];
        let close = values[i * 4 + 2];
        let volume = values[i * 4 + 3];

        let mfv = ((close - low) - (high - close)) / (high - low) * volume;
        money_flow_volumes.push(mfv);
    }

    // Calculate Fast and Slow EMAs of Money Flow Volume
    let fast_ema = calculate_ema(&money_flow_volumes, fast_period)?;
    let slow_ema = calculate_ema(&money_flow_volumes, slow_period)?;

    // The Klinger Oscillator is the difference between the Fast and Slow EMAs
    let klinger_osc = fast_ema - slow_ema;

    Ok(FunctionResult::UnnamedF64(klinger_osc))
}

// Helper function to calculate the Exponential Moving Average (EMA)
fn calculate_ema(values: &[f64], period: usize) -> Result<f64, String> {
    if values.len() < period {
        return Err("Insufficient data for EMA calculation".to_string());
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = values[0];

    for &value in &values[1..] {
        ema = (value - ema) * multiplier + ema;
    }

    Ok(ema)
}

pub fn choppiness_index(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLC data
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period * 4 {
        return Err("Insufficient data for Choppiness Index calculation".to_string());
    }

    let highest_high = values
        .iter()
        .take(period)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let lowest_low = values
        .iter()
        .take(period)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let _range = highest_high - lowest_low;

    let choppiness = 100.0 * (period as f64 / (highest_high - lowest_low));
    Ok(FunctionResult::UnnamedF64(choppiness))
}
