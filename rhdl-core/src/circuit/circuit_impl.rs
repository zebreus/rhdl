use crate::{Digital, DigitalFn};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

/// Function that maps the input ([`CircuitIO::I`]) and state ([`Circuit::Q`]) to the output ([`CircuitIO::O`]) and new state ([`Circuit::D`])
pub type CircuitUpdateFn<C> =
    fn(<C as CircuitIO>::I, <C as Circuit>::Q) -> (<C as CircuitIO>::O, <C as Circuit>::D);

/// Types of HDLs that can be generated.
///
/// Currently only Verilog is supported.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HDLKind {
    Verilog,
}

pub trait Tristate: Default + Clone + Copy {
    const N: usize;
}

impl Tristate for () {
    const N: usize = 0;
}

pub trait CircuitIO: 'static + Sized + Clone {
    type I: Digital;
    type O: Digital;
}

pub struct NoUpdateFn {}

impl DigitalFn for NoUpdateFn {}

/// A circuit
///
/// This is a drawing of the circuit dfg construction
///
///          +--------------------+
///   -----> In                Out >-------->
///          |        update      |
///     +--> Q                   D >-+
///     |    |                    |  |
///     |    +--------------------+  |
///     |                            |
///     +--< Out   child 0      In <-+
///     |                            |
///     +--< Out    child 1     In <-+
///
///  We create buffer nodes for the input and output, D and Q
///  and then connect the update DFG to these node.  The
///  children DFGs are then connected to the D and Q nodes
///  using recursion.

// Create a schematic of the circuit.  It is modified by adding
// a Q buffer and a D buffer.
//          +--------------------+
//   *in ---> In                Out >------*out
//          |        update      |
//     +--> Q                   D >-+
//     |    |                    |  |
//     |    +--------------------+  |
//     |                            |
//     +--< Out   child 0      In <-+
//     |                            |
//     +--< Out    child 1     In <-+
pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    /// Type for the next state
    type D: Digital;
    /// Type for the current state
    type Q: Digital;

    // auto derived as the sum of NumZ of the children
    type Z: Tristate;

    /// Update function for the circuit
    ///
    /// TODO: Figure out when this is called
    type Update: DigitalFn;

    /// TODO: Default update functions?
    ///
    /// Implicit default is |_, _| (Default::default(), Default::default());
    const UPDATE: CircuitUpdateFn<Self>;

    /// State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    /// Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    /// auto derived
    fn name(&self) -> &'static str;

    /// auto derived
    fn descriptor(&self) -> CircuitDescriptor;

    /// auto derived
    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor>;

    /// auto derived
    /// First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
