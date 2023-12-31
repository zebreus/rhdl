use crate::{
    compile_design, digital_fn::KernelFnKind, generate_verilog, kernel::ExternalKernelDef, Digital,
    DigitalFn,
};
use anyhow::bail;
use anyhow::Result;

pub trait Testable<Args, T1> {
    fn test_string(&self, name: &str, args: Args) -> String;
}

impl<F, Q, T0> Testable<(T0,), Q> for F
where
    F: Fn(T0) -> Q,
    T0: Digital,
    Q: Digital,
{
    fn test_string(&self, name: &str, args: (T0,)) -> String {
        let (t0,) = args;
        let q = (*self)(t0).binary_string();
        let t0 = t0.binary_string();
        let t0_bits = T0::static_kind().bits();
        let q_bits = Q::static_kind().bits();
        format!("$display(\"0x%0h 0x%0h\", {q_bits}'b{q}, {name}({t0_bits}'b{t0}));\n")
    }
}

impl<F, Q, T0, T1> Testable<(T0, T1), Q> for F
where
    F: Fn(T0, T1) -> Q,
    T0: Digital,
    T1: Digital,
    Q: Digital,
{
    fn test_string(&self, name: &str, args: (T0, T1)) -> String {
        let (t0, t1) = args;
        let q = (*self)(t0, t1).binary_string();
        let t0 = t0.binary_string();
        let t0_bits = T0::static_kind().bits();
        let t1 = t1.binary_string();
        let t1_bits = T1::static_kind().bits();
        let q_bits = Q::static_kind().bits();
        format!(
            "$display(\"0x%0h 0x%0h\", {q_bits}'b{q}, {name}({t0_bits}'b{t0},{t1_bits}'b{t1}));\n"
        )
    }
}

impl<F, Q, T0, T1, T2> Testable<(T0, T1, T2), Q> for F
where
    F: Fn(T0, T1, T2) -> Q,
    T0: Digital,
    T1: Digital,
    T2: Digital,
    Q: Digital,
{
    fn test_string(&self, name: &str, args: (T0, T1, T2)) -> String {
        let (t0, t1, t2) = args;
        let q = (*self)(t0, t1, t2).binary_string();
        let t0 = t0.binary_string();
        let t0_bits = T0::static_kind().bits();
        let t1 = t1.binary_string();
        let t1_bits = T1::static_kind().bits();
        let t2 = t2.binary_string();
        let t2_bits = T2::static_kind().bits();
        let q_bits = Q::static_kind().bits();
        format!(
            "$display(\"0x%0h 0x%0h\", {q_bits}'b{q}, {name}({t0_bits}'b{t0},{t1_bits}'b{t1},{t2_bits}'b{t2}));\n"
        )
    }
}

impl<F, Q, T0, T1, T2, T3> Testable<(T0, T1, T2, T3), Q> for F
where
    F: Fn(T0, T1, T2, T3) -> Q,
    T0: Digital,
    T1: Digital,
    T2: Digital,
    T3: Digital,
    Q: Digital,
{
    fn test_string(&self, name: &str, args: (T0, T1, T2, T3)) -> String {
        let (t0, t1, t2, t3) = args;
        let q = (*self)(t0, t1, t2, t3).binary_string();
        let t0 = t0.binary_string();
        let t0_bits = T0::static_kind().bits();
        let t1 = t1.binary_string();
        let t1_bits = T1::static_kind().bits();
        let t2 = t2.binary_string();
        let t2_bits = T2::static_kind().bits();
        let t3 = t3.binary_string();
        let t3_bits = T3::static_kind().bits();
        let q_bits = Q::static_kind().bits();
        format!(
            "$display(\"0x%0h 0x%0h\", {q_bits}'b{q}, {name}({t0_bits}'b{t0},{t1_bits}'b{t1},{t2_bits}'b{t2},{t3_bits}'b{t3}));\n"
        )
    }
}

impl<F, Q, T0, T1, T2, T3, T4> Testable<(T0, T1, T2, T3, T4), Q> for F
where
    F: Fn(T0, T1, T2, T3, T4) -> Q,
    T0: Digital,
    T1: Digital,
    T2: Digital,
    T3: Digital,
    T4: Digital,
    Q: Digital,
{
    fn test_string(&self, name: &str, args: (T0, T1, T2, T3, T4)) -> String {
        let (t0, t1, t2, t3, t4) = args;
        let q = (*self)(t0, t1, t2, t3, t4).binary_string();
        let t0 = t0.binary_string();
        let t0_bits = T0::static_kind().bits();
        let t1 = t1.binary_string();
        let t1_bits = T1::static_kind().bits();
        let t2 = t2.binary_string();
        let t2_bits = T2::static_kind().bits();
        let t3 = t3.binary_string();
        let t3_bits = T3::static_kind().bits();
        let t4 = t4.binary_string();
        let t4_bits = T4::static_kind().bits();
        let q_bits = Q::static_kind().bits();
        format!(
            "$display(\"0x%0h 0x%0h\", {q_bits}'b{q}, {name}({t0_bits}'b{t0},{t1_bits}'b{t1},{t2_bits}'b{t2},{t3_bits}'b{t3},{t4_bits}'b{t4}));\n"
        )
    }
}

fn test_module<F, Args, T0>(
    uut: F,
    desc: VerilogDescriptor,
    vals: impl Iterator<Item = Args>,
) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Digital,
{
    let VerilogDescriptor { name, body } = desc;
    let mut num_cases = 0;
    let cases = vals
        .map(|x| {
            num_cases += 1;
            x
        })
        .map(|arg| uut.test_string(&name, arg))
        .collect::<String>();
    TestModule {
        testbench: format!(
            "
module testbench;
   {body}

   initial
       begin
{cases}
$finish;
       end
endmodule
    "
        ),
        num_cases,
    }
}

pub struct VerilogDescriptor {
    pub name: String,
    pub body: String,
}

impl std::fmt::Display for VerilogDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body.fmt(f)
    }
}

pub struct TestModule {
    pub testbench: String,
    pub num_cases: usize,
}

impl TestModule {
    pub fn new<F, Args, T0>(
        uut: F,
        desc: VerilogDescriptor,
        vals: impl Iterator<Item = Args>,
    ) -> TestModule
    where
        F: Testable<Args, T0>,
        T0: Digital,
    {
        test_module(uut, desc, vals)
    }

    pub fn new_from_kernel<K, F, Args, T0>(
        uut: F,
        vals: impl Iterator<Item = Args>,
    ) -> Result<TestModule>
    where
        F: Testable<Args, T0>,
        T0: Digital,
        K: DigitalFn,
    {
        let design = compile_design(K::kernel_fn().try_into()?)?;
        let verilog = generate_verilog(&design)?;
        Ok(test_module(uut, verilog, vals))
    }
}

impl std::fmt::Display for TestModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.testbench.fmt(f)
    }
}

#[cfg(feature = "iverilog")]
impl TestModule {
    pub fn run_iverilog(&self) -> anyhow::Result<()> {
        let d = tempfile::tempdir()?;
        // Write the test bench to a file
        let d_path = d.path();
        std::fs::write(d_path.join("testbench.v"), &self.testbench)?;
        // Compile the test bench
        let mut cmd = std::process::Command::new("iverilog");
        cmd.arg("-o")
            .arg(d_path.join("testbench"))
            .arg(d_path.join("testbench.v"));
        let status = cmd.status()?;
        if !status.success() {
            bail!("Failed to compile testbench");
        }
        let mut cmd = std::process::Command::new(d_path.join("testbench"));
        let output = cmd.output()?;
        for case in String::from_utf8_lossy(&output.stdout)
            .lines()
            .take(self.num_cases)
            .map(|line| line.split(' ').collect::<Vec<_>>())
        {
            let expected = case[0];
            let actual = case[1];
            if case[0] != case[1] {
                bail!("Expected {} but got {}", expected, actual);
            }
        }
        eprintln!("iverilog test passed {} cases OK", self.num_cases);
        Ok(())
    }
}

// This is split up so that in the future we can add additional
// test programs (verilator?) and still keep the back end in place.
#[cfg(feature = "iverilog")]
pub fn test_with_iverilog<F, Args, T0>(
    uut: F,
    desc: VerilogDescriptor,
    vals: impl Iterator<Item = Args>,
) -> anyhow::Result<()>
where
    F: Testable<Args, T0>,
    T0: Digital,
{
    test_module(uut, desc, vals).run_iverilog()
}

impl TryFrom<KernelFnKind> for VerilogDescriptor {
    type Error = anyhow::Error;

    fn try_from(value: KernelFnKind) -> Result<Self, Self::Error> {
        match value {
            KernelFnKind::Extern(ExternalKernelDef { name, body, .. }) => Ok(Self { name, body }),
            _ => bail!("Cannot convert kernel function to Verilog descriptor"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::digital_fn::DigitalFn;
    use rhdl_bits::{alias::*, bits};

    use super::*;
    use itertools::Itertools;
    use rhdl_bits::Bits;

    fn xor<const N: usize>(x: Bits<N>) -> bool {
        let mut x = x.0;
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x & 1 == 1
    }

    #[allow(non_camel_case_types)]
    struct xor<const N: usize> {}

    impl<const N: usize> DigitalFn for xor<N> {
        fn kernel_fn() -> KernelFnKind {
            KernelFnKind::Extern(ExternalKernelDef {
                name: format!("xor_{N}"),
                body: format!(
                    "function [{}:0] xor_{N}(input [{}:0] a); xor_{N} = ^a; endfunction",
                    N - 1,
                    N - 1
                ),
                vm_stub: None,
            })
        }
    }

    fn add(a: b4, b: b4) -> b4 {
        a + b
    }

    #[allow(non_camel_case_types)]
    struct add {}

    impl DigitalFn for add {
        fn kernel_fn() -> KernelFnKind {
            KernelFnKind::Extern(ExternalKernelDef {
                name: "add".to_string(),
                body: "function [3:0] add(input [3:0] a, input [3:0] b); add = a + b; endfunction"
                    .to_string(),
                vm_stub: None,
            })
        }
    }

    #[test]
    fn test_add() -> anyhow::Result<()> {
        let nibbles_a = (0..=15).map(bits);
        let nibbles_b = nibbles_a.clone();
        let module = TestModule::new(
            add,
            add::kernel_fn().try_into()?,
            nibbles_a.cartesian_product(nibbles_b),
        );
        eprintln!("{module}");
        #[cfg(feature = "iverilog")]
        module.run_iverilog()
    }

    #[test]
    fn test_xor_generic() -> anyhow::Result<()> {
        let nibbles_a = (0..=15).map(bits);
        let module = TestModule::new(
            xor::<4>,
            xor::<4>::kernel_fn().try_into()?,
            nibbles_a.map(|x| (x,)),
        );
        eprintln!("{module}");
        #[cfg(feature = "iverilog")]
        module.run_iverilog()
    }
}
