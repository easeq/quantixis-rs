use crate::jit::JITCompiler;
use cranelift::codegen::ir::Function;
use cranelift::codegen::isa::CallConv;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataId, FuncId, Linkage, Module};
use log::debug;
use std::collections::HashMap;

pub struct JITCompilerBuilder {
    functions: Vec<(String, *const u8, Vec<AbiParam>, Vec<AbiParam>)>,
    vars: Vec<(String, *const u8)>,
}

impl JITCompilerBuilder {
    // Create a new builder
    pub fn new() -> Self {
        JITCompilerBuilder {
            functions: Vec::new(),
            vars: Vec::new(),
        }
    }

    // Add a function to the function map
    pub fn add_function(
        mut self,
        name: String,
        ptr: *const u8,
        params: Vec<AbiParam>,
        returns: Vec<AbiParam>,
    ) -> Self {
        self.functions.push((name, ptr, params, returns));
        self
    }

    // Add a variable to the variable map
    pub fn add_variable(mut self, name: String, ptr: *const u8) -> Self {
        self.vars.push((name, ptr));
        self
    }

    pub fn build_data(&self, module: &mut JITModule) -> Result<HashMap<String, DataId>, String> {
        let mut data_map = HashMap::new();

        for (var_name, _ptr) in self.vars.iter() {
            let data_id = module
                .declare_data(&var_name, Linkage::Import, false, false)
                .map_err(|e| e.to_string())?;
            debug!("var_id: {:?}", data_id);

            data_map.insert(var_name.to_string(), data_id);
        }

        Ok(data_map)
    }

    pub fn build_funcs(&self, module: &mut JITModule) -> Result<HashMap<String, FuncId>, String> {
        let mut functions_map = HashMap::new();
        for (func_name, _ptr, params, returns) in self.functions.iter() {
            let mut context = FunctionBuilderContext::new();
            let mut function = Function::new();

            let signature = Signature {
                call_conv: CallConv::SystemV,
                params: params.to_vec(),
                returns: returns.to_vec(),
            };

            function.signature = signature;

            let func_id = module
                .declare_function(&func_name, Linkage::Import, &function.signature)
                .map_err(|e| e.to_string())?;

            debug!("func_id: {func_id:?}");
            debug!("func: {function:#?}");

            let _builder = FunctionBuilder::new(&mut function, &mut context);
            functions_map.insert(func_name.to_string(), func_id);
        }

        Ok(functions_map)
    }

    // Build the JITCompiler with additional logic
    pub fn build(self) -> Result<JITCompiler, String> {
        // Create the JITModule during the build process
        let mut builder = match JITBuilder::new(cranelift_module::default_libcall_names()) {
            Ok(b) => b,
            Err(_) => return Err("Failed to create JITBuilder.".to_string()),
        };

        for (func_name, ptr, _, _) in self.functions.iter() {
            builder.symbol(func_name, ptr.clone());
        }
        builder.symbols(self.vars.clone());

        let mut module = JITModule::new(builder);
        let functions_map = self.build_funcs(&mut module)?;
        let data_map = self.build_data(&mut module)?;

        Ok(JITCompiler {
            module,
            stack: Vec::new(),
            functions_map,
            data_map,
        })
    }
}
