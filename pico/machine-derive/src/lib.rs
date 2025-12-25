use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(MachineDumper)]
pub fn derive_machine_dumper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = input.ident;

    let fields = match input.data {
        Data::Struct(s) => s.fields,
        _ => {
            return syn::Error::new_spanned(
                struct_ident,
                "MachineDumper can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut dump_calls = Vec::new();

    match fields {
        Fields::Named(named) => {
            for ident in named.named.iter().map(|f| f.ident.as_ref().unwrap()) {
                if !dump_calls.is_empty() {
                    dump_calls.push(quote! {
                        ret.push_str(" ");
                    });
                }
                dump_calls.push(quote! {
                    ret.push_str(&self.#ident.x_dump());
                });
            }
        }
        _ => {
            return syn::Error::new_spanned(
                struct_ident,
                "MachineDumper currently supports only named-field structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        impl MachineDumper for #struct_ident {
            fn x_dump(&self) -> String {
                let mut ret = String::new();
                #(#dump_calls)*
                ret
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(MachineParser)]
pub fn derive_machine_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = input.ident;

    let fields = match input.data {
        Data::Struct(s) => s.fields,
        _ => {
            return syn::Error::new_spanned(
                struct_ident,
                "MachineParser can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut parse_calls = Vec::new();

    match fields {
        Fields::Named(named) => {
            for ident in named.named.iter().map(|f| f.ident.as_ref().unwrap()) {
                parse_calls.push(quote! {
                    self.#ident.x_parse(tokens)?;
                });
            }
        }
        _ => {
            return syn::Error::new_spanned(
                struct_ident,
                "MachineDumper currently supports only named-field structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        impl MachineParser for #struct_ident {
            fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
                #(#parse_calls)*
                Ok(())
            }
        }
    };

    expanded.into()
}
