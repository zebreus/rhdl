use std::collections::HashMap;

use crate::{root_verilog, Circuit, HDLKind};

/// Represents a module in a specific target HDL.
///
/// For now it only supports Verilog, but in the future it could support other HDLs.
#[derive(Clone, Debug)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: String,
    pub children: HashMap<String, HDLDescriptor>,
}

impl std::fmt::Display for HDLDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.body)?;
        for hdl in self.children.values() {
            writeln!(f, "{}", hdl)?;
        }
        Ok(())
    }
}

impl HDLDescriptor {
    pub fn add_child<C: Circuit>(
        &mut self,
        name: &str,
        circuit: &C,
        kind: HDLKind,
    ) -> anyhow::Result<()> {
        self.children.insert(name.into(), circuit.as_hdl(kind)?);
        Ok(())
    }
}

/// Converts a RHDL circuit into a module in a specific target HDL. For now only Verilog is supported.
///
/// TODO: What is the differnce between this and `as_hdl`?
/// # Arguments
///
/// * `circuit` - The circuit to generate the HDL descriptor for.
/// * `kind` - The target HDL
///
///
pub fn root_hdl<C: Circuit>(circuit: &C, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
    match kind {
        HDLKind::Verilog => root_verilog(circuit),
    }
}
