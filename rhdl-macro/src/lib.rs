use proc_macro::TokenStream;

/// Derive the [Digital] and the [rhdl::Notable] trait for a struct.
#[proc_macro_derive(Digital, attributes(rhdl))]
pub fn digital(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_digital(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Kernel macro
///
/// Creates a struct with the same name as the function that implements the [DigitalFn] trait.
#[proc_macro_attribute]
pub fn kernel(_attr: TokenStream, input: TokenStream) -> TokenStream {
    match rhdl_macro_core::hdl_kernel(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn hdl(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::hdl_kernel(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Circuit, attributes(rhdl))]
pub fn circuit(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_circuit(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
