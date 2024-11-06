use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

#[proc_macro_attribute]
pub fn modularize(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let mut match_arms = Vec::new();

    for variant in &input.variants {
        let variant_name = &variant.ident;
        match_arms.push(quote! {
            #enum_name::#variant_name(x) => <#variant_name as Module<SAMPLE_RATE>>::process::<POINTS>(x, patchbay),
        });
    }

    let gen = quote! {
        #input

        impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for #enum_name {
            fn process<const POINTS: usize>(&mut self, patchbay: &mut Patchbay<POINTS>) -> Result<(), PatchError> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    gen.into()
}
