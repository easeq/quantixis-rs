use log::debug;
use std::collections::HashMap;

/// The runtime environment holds the variable map and a contiguous data array.
/// Each variable is represented by a 64-bit slot.
/// (For non‑integer types you can convert the value to a 64‑bit representation.)
#[derive(Debug)]
pub struct RuntimeEnvironment {
    /// Maps variable names to their slot index in `data`.
    map: HashMap<String, usize>,
    /// A contiguous array holding one 64-bit word per variable.
    data: Vec<i64>,
    ptr: *mut u64,
}

impl RuntimeEnvironment {
    /// Create a new environment given a list of variable names.
    pub fn new(var_names: &[String]) -> Self {
        let mut map = HashMap::new();
        for (i, name) in var_names.iter().enumerate() {
            map.insert(name.to_string(), i);
        }

        Self {
            map,
            data: vec![0i64; var_names.len()],
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn set_i64(&mut self, name: &str, value: i64) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value;
        }
    }

    pub fn set_i32(&mut self, name: &str, value: i32) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value as i64;
        }
    }

    /// Set a floating‑point value (by storing its bit pattern).
    pub fn set_f64(&mut self, name: &str, value: f64) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value.to_bits() as i64;
        }
    }

    /// For types such as arrays or hashmaps, store the pointer value.
    pub fn set_ptr<T>(&mut self, name: &str, ptr: *mut T) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = ptr as i64;
        }
    }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value as i64;
        }
    }

    pub fn init(&mut self) {
        let buffer = self.data.clone().into_boxed_slice(); // Allocate space for variables
        let ptr = if self.data.len() > 0 {
            Box::into_raw(buffer) as *mut u64
        } else {
            std::ptr::null_mut()
        };

        self.ptr = ptr;
    }

    /// Returns a mutable pointer to the underlying contiguous data.
    pub fn as_ptr(&mut self) -> *mut u64 {
        self.ptr
    }
}

impl Drop for RuntimeEnvironment {
    fn drop(&mut self) {
        debug!("drop RuntimeEnvironment");
        if !self.ptr.is_null() {
            let cur = unsafe { Box::from_raw(self.ptr) };
            drop(cur);
        }
    }
}
