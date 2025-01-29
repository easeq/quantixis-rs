use crate::ast::{Executor, Value};

pub fn register(executor: &mut Executor) {
    // executor.register_function("average_true_range", average_true_range);
    // // executor.register_function("bollinger_bands", bollinger_bands);
}

// // pub fn bollinger_bands(args: &FunctionArgs) -> Result<FunctionResult, String> {
// //     let values = args.get_array("values").unwrap_or(&[]);
// //     let period = args.get_number("period").unwrap_or(20.0) as usize;
// //     let multiplier = args.get_number("multiplier").unwrap_or(2.0);
// //
// //     if values.len() < period {
// //         return Err("Insufficient data for Bollinger Bands calculation".to_string());
// //     }
// //
// //     // Calculate the SMA (middle band)
// //     let middle_band = simple_moving_average(&args)?;
// //
// //     // Calculate standard deviation
// //     let variance = values
// //         .iter()
// //         .take(period)
// //         .map(|x| (*x - middle_band).powi(2))
// //         .sum::<f64>()
// //         / period as f64;
// //     let std_dev = variance.sqrt();
// //
// //     // Calculate the upper and lower bands
// //     let upper_band = middle_band + multiplier * std_dev;
// //     let lower_band = middle_band - multiplier * std_dev;
// //
// //     Ok(FunctionResult::NamedF64Map(HashMap::from([
// //         ("upper_band", upper_band),
// //         ("middle_band", middle_band),
// //         ("lower_band", lower_band),
// //     ])))
// // }
//
// pub fn average_true_range(args: &FunctionArgs) -> Result<FunctionResult, String> {
//     let values = args.get_array("values").unwrap_or(&[]);
//     let period = args.get_number("period").unwrap_or(14.0) as usize;
//
//     if values.len() < period + 1 {
//         return Err("Insufficient data for the specified period".to_string());
//     }
//
//     let true_ranges: Vec<f64> = values
//         .windows(2)
//         .map(|pair| (pair[1] - pair[0]).abs())
//         .collect();
//
//     let atr = true_ranges.iter().take(period).sum::<f64>() / period as f64;
//     Ok(FunctionResult::UnnamedF64(atr))
// }
//
// pub fn stddev(data: &[f64]) -> f64 {
//     if data.is_empty() {
//         return 0.0;
//     }
//     let mean = data.iter().sum::<f64>() / data.len() as f64;
//     let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / data.len() as f64;
//     variance.sqrt()
// }
