//! This module exports the main entrypoints for the compiler.
//!
//! To compile a kernel function into a verilog module, use `compile_design`.
//!
//! ## `compile_design` overview
//!
//! 1. Get an [`Object`] from the kernel function using [`compile_kernel`].
//! 2. Create a [`Module`] with the object as the top level object.
//! 3. Find all external kernels in the module, compile them, and add them to the module.
//! 4. Repeat step 3 until no new external kernels are added.
//! 5. Return the generated module.
//!
//! The module is now ready to be rendered into verilog. See [`crate::generate_verilog`] for that
//!
//! ## Objects relevant to the compiler
//!
//! - [`Kernel`]: The main data structure representing a uncompiled kernel function.
//! - [`Object`]: The main data structure representing a compiled kernel function. Basically contains all information needed to generate verilog.
//! - [`Module`]: A collection of objects. The top level object is the object that will be the top level module in the generated verilog.
//!

use crate::{
    compiler::{
        ascii::render_ast_to_string, assign_node_ids, check_inference::check_inference,
        check_rhif_flow::DataFlowCheckPass, check_rhif_type::TypeCheckPass, compile, infer,
        pass::Pass, pre_cast_literals::PreCastLiterals,
        remove_extra_registers::RemoveExtraRegistersPass,
        remove_unneeded_muxes::RemoveUnneededMuxesPass,
        remove_unused_literals::RemoveUnusedLiterals, remove_useless_casts::RemoveUselessCastsPass,
    },
    kernel::Kernel,
    rhif::{spec::ExternalFunctionCode, Object},
    Module,
};

use anyhow::Result;

/// Compile [`Kernel`] into an [`Object`].
///
/// This function will compile the kernel, infer types, and run a series of optimization passes.
pub fn compile_kernel(mut kernel: Kernel) -> Result<Object> {
    assign_node_ids(&mut kernel)?;
    let ctx = infer(&kernel)?;
    let _ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
    check_inference(&kernel, &ctx)?;
    let mut obj = compile(kernel.inner(), ctx)?;
    eprintln!("{}", obj);
    for _pass in 0..2 {
        obj = RemoveExtraRegistersPass::run(obj)?;
        obj = RemoveUnneededMuxesPass::run(obj)?;
        obj = RemoveExtraRegistersPass::run(obj)?;
        obj = RemoveUnusedLiterals::run(obj)?;
        obj = PreCastLiterals::run(obj)?;
        obj = RemoveUselessCastsPass::run(obj)?;
    }
    let obj = TypeCheckPass::run(obj)?;
    let obj = DataFlowCheckPass::run(obj)?;
    Ok(obj)
}

/// Find and compile all uncompiled external kernels in the module.
fn elaborate_design(design: &mut Module) -> Result<()> {
    // Find all external kernels
    let external_kernels = design
        .objects
        .values()
        .flat_map(|obj| obj.externals.iter())
        .filter_map(|func| {
            if let ExternalFunctionCode::Kernel(kernel) = &func.code {
                Some(kernel)
            } else {
                None
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    // Compile any uncompiled external kernels
    for kernel in external_kernels {
        if let std::collections::hash_map::Entry::Vacant(e) =
            design.objects.entry(kernel.inner().fn_id)
        {
            eprintln!("Compiling kernel {}", kernel.inner().fn_id);
            let obj = compile_kernel(kernel.clone())?;
            e.insert(obj);
        }
    }
    Ok(())
}

/// Compile a top level kernel and all its dependencies into a Verilog module.
pub fn compile_design(top: Kernel) -> Result<Module> {
    // Create a design from the top level kernel
    let main = compile_kernel(top)?;
    let mut design: Module = Module {
        objects: [(main.fn_id, main.clone())].into_iter().collect(),
        top: main.fn_id,
    };

    // Elaborate the design until no new objects are added
    let mut object_count = design.objects.len();
    loop {
        elaborate_design(&mut design)?;
        if design.objects.len() == object_count {
            break;
        }
        object_count = design.objects.len();
    }
    Ok(design)
}
