use log::debug;
use std::collections::HashMap;
use std::mem;
use std::ptr;

/// Helper struct for representing array metadata. When setting an array variable,
/// we allocate one of these on the heap and store its pointer (as i64) in the environment.
#[repr(C)]
pub struct ArrayMeta {
    pub ptr: *mut (), // pointer to the actual array data
    pub len: usize,
    pub capacity: usize,
}

impl ArrayMeta {
    pub fn new<T>(boxed_slice: Box<[T]>) -> Self {
        let len = boxed_slice.len();
        // In this example we assume capacity equals length.
        // (You might change this if using Vec.)
        let capacity = len;
        // Convert the boxed slice into a raw pointer.
        // Note that we lose the type information.
        let ptr = Box::into_raw(boxed_slice) as *mut ();
        Self { ptr, len, capacity }
    }
}

/// The runtime environment holds the variable map and a contiguous data array.
/// Each variable is represented by one 64-bit slot (an i64). For numbers and booleans,
/// the value is stored directly; for arrays/hashmaps, we store a pointer (and you can later
/// recover metadata such as length/capacity using helper functions).
#[derive(Debug)]
pub struct RuntimeEnvironment {
    /// Maps variable names to their slot index in `data`.
    pub map: HashMap<String, usize>,
    /// A contiguous array holding one 64-bit word per variable.
    pub data: Vec<i64>,
    /// A pointer to a heap‑allocated contiguous block. This pointer is updated on `init()`.
    pub ptr: *mut u64,
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
            ptr: ptr::null_mut(),
        }
    }

    /// Set an i64 value.
    pub fn set_i64(&mut self, name: &str, value: i64) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value;
        }
    }

    /// Set an i32 value.
    pub fn set_i32(&mut self, name: &str, value: i32) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value as i64;
        }
    }

    /// Set an f64 value by storing its bit pattern.
    pub fn set_f64(&mut self, name: &str, value: f64) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value.to_bits() as i64;
        }
    }

    /// Set an f32 value by storing its bit pattern.
    pub fn set_f32(&mut self, name: &str, value: f32) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = value.to_bits() as u64 as i64;
        }
    }

    /// Set a boolean value (true → 1, false → 0).
    pub fn set_bool(&mut self, name: &str, value: bool) {
        if let Some(&index) = self.map.get(name) {
            self.data[index] = if value { 1 } else { 0 };
        }
    }

    /// Set an array of i32 values by storing an ArrayMeta pointer.
    pub fn set_array_i32(&mut self, name: &str, arr: Box<[i32]>) {
        if let Some(&index) = self.map.get(name) {
            let meta = ArrayMeta::new(arr);
            let ptr_val = Box::into_raw(Box::new(meta)) as *mut () as i64;
            self.data[index] = ptr_val;
        }
    }

    /// Set an array of i64 values.
    pub fn set_array_i64(&mut self, name: &str, arr: Box<[i64]>) {
        if let Some(&index) = self.map.get(name) {
            let meta = ArrayMeta::new(arr);
            let ptr_val = Box::into_raw(Box::new(meta)) as *mut () as i64;
            self.data[index] = ptr_val;
        }
    }

    /// Set an array of f32 values.
    pub fn set_array_f32(&mut self, name: &str, arr: Box<[f32]>) {
        if let Some(&index) = self.map.get(name) {
            let meta = ArrayMeta::new(arr);
            let ptr_val = Box::into_raw(Box::new(meta)) as *mut () as i64;
            self.data[index] = ptr_val;
        }
    }

    /// Set an array of f64 values.
    pub fn set_array_f64(&mut self, name: &str, arr: Box<[f64]>) {
        if let Some(&index) = self.map.get(name) {
            let meta = ArrayMeta::new(arr);
            let ptr_val = Box::into_raw(Box::new(meta)) as *mut () as i64;
            self.data[index] = ptr_val;
        }
    }

    /// Set a generic array (of any type) by storing its pointer in an ArrayMeta.
    pub fn set_generic_array<T: 'static>(&mut self, name: &str, arr: Box<[T]>) {
        if let Some(&index) = self.map.get(name) {
            let meta = ArrayMeta::new(arr);
            let ptr_val = Box::into_raw(Box::new(meta)) as *mut () as i64;
            self.data[index] = ptr_val;
        }
    }

    /// Set a generic hashmap by storing its pointer.
    pub fn set_generic_hashmap<K: 'static, V: 'static>(
        &mut self,
        name: &str,
        map_val: Box<HashMap<K, V>>,
    ) {
        if let Some(&index) = self.map.get(name) {
            let ptr_val = Box::into_raw(map_val) as *mut () as i64;
            self.data[index] = ptr_val;
        }
    }

    /// Initialize the environment by (re)allocating a contiguous memory block for the data.
    /// If a block was previously allocated (self.ptr is not null), drop it first.
    pub fn init(&mut self) {
        // If self.ptr already points to an allocated block, drop it.
        if !self.ptr.is_null() {
            unsafe {
                // Convert the raw pointer back into a Box to drop it.
                let _ = Box::from_raw(self.ptr);
            }
        }
        let buffer = self.data.clone().into_boxed_slice();
        self.ptr = if self.data.len() > 0 {
            Box::into_raw(buffer) as *mut u64
        } else {
            ptr::null_mut()
        };
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
            unsafe {
                let _ = Box::from_raw(self.ptr);
            }
        }
    }
}
