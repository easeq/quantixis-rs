use crate::ast::{FunctionArgs, FunctionResult};
use crate::Evaluator;

pub fn register(evaluator: &mut Evaluator) {
    evaluator.register_function("simple_moving_average", simple_moving_average);
    evaluator.register_function("exponential_moving_average", exponential_moving_average);
    evaluator.register_function("relative_strength_index", relative_strength_index);
    evaluator.register_function(
        "moving_average_convergence_divergence",
        moving_average_convergence_divergence,
    );
    evaluator.register_function("ichimoku_cloud", ichimoku_cloud);
    evaluator.register_function("parabolic_sar", parabolic_sar);
}

pub fn simple_moving_average(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for the specified period".to_string());
    }

    let sma = values.iter().take(period).sum::<f64>() / period as f64;
    Ok(FunctionResult::UnnamedF64(sma))
}

pub fn exponential_moving_average(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for the specified period".to_string());
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = values[0];

    for &value in &values[1..] {
        ema = (value - ema) * multiplier + ema;
    }

    Ok(FunctionResult::UnnamedF64(ema))
}

pub fn relative_strength_index(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let period = args.get_number("period").unwrap_or(14.0) as usize;

    if values.len() < period {
        return Err("Insufficient data for the specified period".to_string());
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let change = values[i] - values[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses -= change;
        }
    }

    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;

    if avg_loss == 0.0 {
        return Ok(FunctionResult::UnnamedF64(100.0)); // No losses, RSI is 100
    }

    let rs = avg_gain / avg_loss;
    Ok(FunctionResult::UnnamedF64(100.0 - (100.0 / (1.0 + rs))))
}

pub fn moving_average_convergence_divergence(
    args: &FunctionArgs,
) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let short_period = args.get_number("short_period").unwrap_or(12.0) as usize;
    let long_period = args.get_number("long_period").unwrap_or(26.0) as usize;
    let signal_period = args.get_number("signal_period").unwrap_or(9.0) as usize;

    if values.len() < long_period {
        return Err("Insufficient data for the specified periods".to_string());
    }

    let short_ema = calculate_ema(values, short_period)?;
    let long_ema = calculate_ema(values, long_period)?;
    let macd = short_ema - long_ema;

    let signal = calculate_ema(&vec![macd], signal_period)?;
    let histogram = macd - signal;

    Ok(FunctionResult::UnnamedF64(histogram))
}

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

pub fn ichimoku_cloud(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]);
    let conversion_period = args.get_number("conversion_period").unwrap_or(9.0) as usize;
    let base_period = args.get_number("base_period").unwrap_or(26.0) as usize;
    let span_b_period = args.get_number("span_b_period").unwrap_or(52.0) as usize;

    if values.len() < span_b_period {
        return Err("Insufficient data for Ichimoku Cloud calculation".to_string());
    }

    let tenkan_sen = calculate_ichimoku_line(values, conversion_period)?;
    let kijun_sen = calculate_ichimoku_line(values, base_period)?;
    let senkou_span_a = (tenkan_sen + kijun_sen) / 2.0;
    let senkou_span_b = calculate_ichimoku_line(values, span_b_period)?;

    Ok(FunctionResult::UnnamedF64(senkou_span_a - senkou_span_b))
}

fn calculate_ichimoku_line(values: &[f64], period: usize) -> Result<f64, String> {
    if values.len() < period {
        return Err("Insufficient data for Ichimoku calculation".to_string());
    }

    let max_high = values.iter().take(period).cloned().fold(f64::MIN, f64::max);
    let min_low = values.iter().take(period).cloned().fold(f64::MAX, f64::min);

    Ok((max_high + min_low) / 2.0)
}

pub fn parabolic_sar(args: &FunctionArgs) -> Result<FunctionResult, String> {
    let values = args.get_array("values").unwrap_or(&[]); // Assumes OHLCV data
    let acceleration_factor = args.get_number("acceleration_factor").unwrap_or(0.02);
    let max_acceleration = args.get_number("max_acceleration").unwrap_or(0.2);

    if values.len() < 2 {
        return Err("Insufficient data for Parabolic SAR calculation".to_string());
    }

    let mut sar = values[0]; // Start with the first value as SAR
    let mut ep = values[1]; // Set the first EP
    let mut af = acceleration_factor; // Start with the initial AF

    // Calculate SAR for each subsequent value
    for i in 1..values.len() {
        let sar_next = sar + af * (ep - sar); // Next SAR value
        sar = sar_next;

        // Check if we need to adjust the direction (reversal)
        if values[i] > ep {
            ep = values[i]; // Update EP for an uptrend
            af = (af + acceleration_factor).min(max_acceleration);
        } else {
            ep = values[i]; // Update EP for a downtrend
            af = (af + acceleration_factor).min(max_acceleration);
        }
    }

    Ok(FunctionResult::UnnamedF64(sar))
}
