use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

synstructure::decl_derive!(
    [Node] => node_derive
);

/// Derives the `Node` for the AST node.
fn node_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match &s.ast().data {
        syn::Data::Struct(_) => node_derive_struct(s).unwrap_or_else(|err| err.to_compile_error()),
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `Node` for Rust `struct` items",
            )
            .to_compile_error()
        }
    }
}

fn node_derive_struct(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    assert_eq!(s.variants().len(), 1, "can only operate on structs");
    if !s.ast().generics.params.is_empty() {
        return Err(syn::Error::new(
            s.ast().generics.params.span(),
            "can only derive `Node` for structs without generics",
        ));
    }
    let ident = &s.ast().ident;

    let mut contains_loc = false;

    let fields: Vec<(syn::Ident, syn::Type)> = s.variants()[0]
        .bindings()
        .iter()
        .filter_map(|info| {
            let ident = info.ast().ident.clone().unwrap();
            if &ident.to_string() == "loc" {
                contains_loc = true;
                return None;
            }
            let ty = &info.ast().ty;
            Some((ident, ty.clone()))
        })
        .collect();

    let params = fields.iter().map(|(i, t)| {
        quote! { #i: #t , }
    });

    let args = fields.iter().map(|(i, _)| {
        quote! { #i, }
    });

    let loc_param = contains_loc.then(|| quote! { start: usize, end: usize, });

    let loc_arg = contains_loc.then(|| quote! { loc: Span { start, end }, });

    Ok(quote! {
        impl #ident {
            #[allow(clippy::too_many_arguments)]
            pub fn new(#loc_param #(#params)*) -> Self {
                Self {
                    #loc_arg
                    #(#args)*
                }
            }
        }
    })
}
