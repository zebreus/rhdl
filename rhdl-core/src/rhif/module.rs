use crate::{ast::ast_impl::FunctionId, rhif::Object};
use anyhow::Result;
use std::collections::HashMap;

use super::spanned_source::SpannedSource;

/// Represents set of Verilog functions and information about the top level function.
///
/// Can be created using [`crate::compile_design`]. Can be vonvert to Verilog using [`crate::generate_verilog`].
///
/// You can also use [`crate::execute_function`] to simulate the top level function.
#[derive(Clone, Debug)]
pub struct Module {
    /// All functions in this module.
    ///
    /// This contains the top level function and all external functions referenced by the top level function.
    pub objects: HashMap<FunctionId, Object>,
    /// ID of the top level function.
    pub top: FunctionId,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Design {}", self.top)?;
        for obj in self.objects.values() {
            write!(f, "\n  Object {}", obj.name)?;
        }
        Ok(())
    }
}

impl Module {
    /// Get the name of a function by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the function ID is not found in this module.
    pub fn func_name(&self, fn_id: FunctionId) -> Result<String> {
        let obj = self
            .objects
            .get(&fn_id)
            .ok_or(anyhow::anyhow!("Function {fn_id} not found"))?;
        Ok(format!("{}_{:x}", obj.name, fn_id))
    }
    /// Get the source maps for all functions in this module.
    pub fn source_map(&self) -> HashMap<FunctionId, SpannedSource> {
        self.objects
            .iter()
            .map(|(fn_id, obj)| (*fn_id, obj.symbols.source.clone()))
            .collect()
    }
}
