#![doc = include_str!("../README.md")]

use heck::ToShoutySnakeCase;
use proc_macro::TokenStream as StdTokenStream;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, Generics, Lifetime, LifetimeParam, LitInt,
    Path, Result, Variant, parse_quote, parse2,
};

mod decode;
mod encode;
mod packet;

#[proc_macro_derive(Encode, attributes(packet))]
pub fn derive_encode(item: StdTokenStream) -> StdTokenStream {
    match encode::derive_encode(item.into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(Decode, attributes(packet))]
pub fn derive_decode(item: StdTokenStream) -> StdTokenStream {
    match decode::derive_decode(item.into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(item: StdTokenStream) -> StdTokenStream {
    match packet::derive_packet(item.into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(RegistryId, attributes(packet))]
pub fn derive_registry_id(item: StdTokenStream) -> StdTokenStream {
    match registry_id(item.into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn registry_id(item: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(item)?;
    let registry = registry_path(&input.attrs)?.ok_or_else(|| {
        syn::Error::new_spanned(&input.ident, "expected #[packet(registry = path)]")
    })?;
    let Data::Enum(data) = input.data else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "RegistryId requires an enum",
        ));
    };
    let count = data.variants.len();
    let variants = pair_variants_with_discriminants(&input.attrs, data.variants)?;
    let arms = variants.iter().map(|(id, variant)| {
        let name = &variant.ident;
        let pattern = match &variant.fields {
            Fields::Named(_) => quote!(Self::#name { .. }),
            Fields::Unnamed(_) => quote!(Self::#name(..)),
            Fields::Unit => quote!(Self::#name),
        };
        quote!(#pattern => #id,)
    });
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        // Every registry entry needs a payload definition before it can be decoded.
        const _: [(); #registry::COUNT] = [(); #count];

        impl #impl_generics #name #ty_generics #where_clause {
            pub const fn id(&self) -> i32 {
                match self { #(#arms)* }
            }
        }
    })
}

fn pair_variants_with_discriminants(
    attrs: &[Attribute],
    variants: impl IntoIterator<Item = Variant>,
) -> Result<Vec<(TokenStream, Variant)>> {
    if let Some(registry) = registry_path(attrs)? {
        return variants
            .into_iter()
            .map(|variant| {
                if parse_tag_attr(&variant.attrs)?.is_some() {
                    return Err(syn::Error::new_spanned(
                        &variant,
                        "registry-tagged enum variants cannot have explicit tags",
                    ));
                }
                let id = format_ident!("{}", variant.ident.to_string().to_shouty_snake_case());
                Ok((quote!(#registry::#id), variant))
            })
            .collect();
    }

    let mut discriminant = 0;
    variants
        .into_iter()
        .map(|v| {
            if let Some(id) = parse_tag_attr(&v.attrs)? {
                discriminant = id;
            }
            let id = discriminant;
            discriminant += 1;
            Ok((quote!(#id), v))
        })
        .collect::<Result<_>>()
}

fn registry_path(attrs: &[Attribute]) -> Result<Option<Path>> {
    let mut registry = None;
    for attr in attrs {
        if attr.path().is_ident("packet") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("registry") {
                    if registry.is_some() {
                        return Err(meta.error("duplicate registry path"));
                    }
                    registry = Some(meta.value()?.parse::<Path>()?);
                    Ok(())
                } else if ["id", "name", "side", "state"]
                    .iter()
                    .any(|name| meta.path.is_ident(name))
                {
                    // These enum-level arguments belong to the Packet derive.
                    // Encode and Decode must continue to accept them as before.
                    meta.value()?.parse::<syn::Expr>()?;
                    Ok(())
                } else {
                    Err(meta.error("unrecognized packet argument"))
                }
            })?;
        }
    }
    Ok(registry)
}

fn parse_tag_attr(attrs: &[Attribute]) -> Result<Option<i32>> {
    for attr in attrs {
        if attr.path().is_ident("packet") {
            let mut res = 0;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("tag") {
                    res = meta.value()?.parse::<LitInt>()?.base10_parse::<i32>()?;
                    Ok(())
                } else {
                    Err(meta.error("unrecognized argument"))
                }
            })?;

            return Ok(Some(res));
        }
    }

    Ok(None)
}

/// Adding our lifetime to the generics before calling `.split_for_impl()` would
/// also add it to the resulting `ty_generics`, which we don't want. So I'm
/// doing this hack.
fn decode_split_for_impl(
    mut generics: Generics,
    lifetime: Lifetime,
) -> (TokenStream, TokenStream, TokenStream) {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut impl_generics = impl_generics.to_token_stream();
    let ty_generics = ty_generics.to_token_stream();
    let where_clause = where_clause.to_token_stream();

    if generics.lifetimes().next().is_none() {
        generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(lifetime)));

        impl_generics = generics.split_for_impl().0.to_token_stream();
    }

    (impl_generics, ty_generics, where_clause)
}

fn add_trait_bounds(generics: &mut Generics, trait_: TokenStream) {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(#trait_))
        }
    }
}
