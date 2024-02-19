mod digital;
mod utils;
pub use digital::derive_digital;
mod digital_enum;
mod kernel;
pub use kernel::hdl_kernel;
mod circuit;
mod suffix;
pub use circuit::derive_circuit;
