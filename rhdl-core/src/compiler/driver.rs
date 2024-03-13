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

pub fn compile_kernel(mut kernel: Kernel) -> Result<Object> {
    assign_node_ids(&mut kernel)?;
    let ctx = infer(&kernel)?;
    let _ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
    //eprintln!("{}", ast_ascii);
    check_inference(&kernel, &ctx)?;
    let mut obj = compile(kernel.inner(), ctx)?;
    //    let obj = LowerIndexToCopy::run(obj)?;
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
    eprintln!("{}", obj);

    /*     if let Some(source) = obj.source.as_ref() {
           obj.opcode_map
               .iter()
               .zip(obj.ops.iter())
               .for_each(|(location, opcode)| {
                   if matches!(
                       opcode,
                       OpCode::Assign(_) | OpCode::Index(_) | OpCode::Splice(_),
                   ) {
                       eprintln!("opcode: {}", opcode);
                       show_source(source, &opcode.to_string(), location.node);
                   }
               });
       }
    */
    /*
    if let Some(source) = obj.source.as_ref() {
        for (reg, loc) in &obj.register_map {
            show_source(source, &reg.to_string(), loc.node);
        }
    }*/
    Ok(obj)
}

fn elaborate_design(design: &mut Module) -> Result<()> {
    // Check for any uncompiled kernels
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

pub fn compile_design(top: Kernel) -> Result<Module> {
    let main = compile_kernel(top)?;
    let mut design = Module {
        objects: [(main.fn_id, main.clone())].into_iter().collect(),
        top: main.fn_id,
    };
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
