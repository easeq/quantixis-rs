mod builder;
mod compiler;
mod functions;
mod rt_env;

pub use builder::JITCompilerBuilder;
pub use compiler::*;
pub use rt_env::*;

use log::debug;

/// **Executes compiled JIT function**
pub fn execute(func_id: *const u8, memory_ptr: *mut u64) -> Result<f64, String> {
    debug!("memory_ptr: {:?}", memory_ptr.is_null());

    let func: extern "C" fn(*mut u64) -> f64 = unsafe { std::mem::transmute(func_id) };
    Ok(func(memory_ptr))
}
