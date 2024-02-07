use anyhow::Result;
use rhdl_core::path::{bit_range, Path};
use rhdl_core::{ast_builder::path, test_module::VerilogDescriptor};
use rhdl_core::{compile_design, generate_verilog, Digital, DigitalFn};
use std::hash::{Hash, Hasher};

use crate::circuit::{Circuit, CircuitDescriptor};
use crate::translator::Translator;

pub struct VerilogTranslator;

impl Translator for VerilogTranslator {
    fn translate<C: Circuit>(&self, t: &C) -> Result<String> {
        // Start with the module declaration for the circuit.
        let descriptor = t.descriptor();
        let input_bits = C::I::bits();
        let outputs = C::O::bits();

        // module top(input wire clk, input wire[0:0] top_in, output reg[3:0] top_out);

        let module_decl = format!(
            "module {module_name}(input wire[{INPUT_BITS}:0] i, output wire[{OUTPUT_BITS}:0] o);",
            module_name = descriptor.unique_name,
            INPUT_BITS = input_bits.saturating_sub(1),
            OUTPUT_BITS = outputs.saturating_sub(1)
        );

        let o_d_bits = C::O::bits() + C::D::bits();
        // Next declare the D and Q wires
        let od_decl = format!(
            "wire[{OD_BITS}:0] od;",
            OD_BITS = o_d_bits.saturating_sub(1)
        );
        let d_decl = format!(
            "wire[{D_BITS}:0] d;",
            D_BITS = C::D::bits().saturating_sub(1)
        );
        let q_decl = format!(
            "wire[{Q_BITS}:0] q;",
            Q_BITS = C::Q::bits().saturating_sub(1)
        );

        // Next, for each subcomponent, we need to determine it's input range from the Q and D types.
        // Loop over the components.
        let component_decls = t
            .components()
            .enumerate()
            .map(|(ndx, (local, desc))| component_decl::<C>(ndx, &local, &desc))
            .collect::<Result<Vec<_>>>()?
            .join("\n");

        let verilog = generate_verilog(&compile_design(C::Update::kernel_fn().try_into()?)?)?;
        let fn_call = format!("assign od = {fn_name}(i, q);", fn_name = &verilog.name);
        let fn_body = &verilog.body;
        let o_bind = format!("assign o = od[{}:{}];", outputs.saturating_sub(1), 0);
        let d_bind = format!(
            "assign d = od[{}:{}];",
            C::D::bits().saturating_sub(1),
            outputs
        );
        Ok(format!(
            "{module_decl}
{od_decl}
{d_decl}
{q_decl}
{o_bind}
{d_bind}

{component_decls}

{fn_call}

{fn_body}

endmodule

",
        ))
    }
}

fn component_decl<C: Circuit>(
    ndx: usize,
    local_name: &str,
    desc: &CircuitDescriptor,
) -> Result<String> {
    // instantiate the component with name components.name.
    // give it a unique instance name of c{ndx}
    // wire the inputs to the range of d that corresponds to the name
    // wire the outputs to the range of q that corresponds to the name
    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();
    let (d_range, _) = bit_range(d_kind, &Path::default().field(local_name))?;
    let (q_range, _) = bit_range(q_kind, &Path::default().field(local_name))?;
    Ok(format!(
        "{component_name} c{ndx} (.i(d[{d_msb}:{d_lsb}]),.o(q[{q_msb}:{q_lsb}]));",
        component_name = desc.unique_name,
        ndx = ndx,
        d_msb = d_range.end.saturating_sub(1),
        d_lsb = d_range.start,
        q_msb = q_range.end.saturating_sub(1),
        q_lsb = q_range.start
    ))
}
