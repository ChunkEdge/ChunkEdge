#![doc = include_str!("../README.md")]

use heck::ToShoutySnakeCase;
use proc_macro::TokenStream as StdTokenStream;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, Generics, Lifetime, LifetimeParam, LitInt,
    LitStr, Path, Result, Variant, parse_quote, parse2,
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
    let (registry, _) = registry_config(&input.attrs)?.ok_or_else(|| {
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
        quote!(#pattern => #id as i32,)
    });
    let checks = variants.iter().map(|(id, _)| {
        quote! {
            let id = #id as usize;
            assert!(id < #registry::COUNT, "registry ID out of range");
            assert!(!seen[id], "duplicate registry ID");
            seen[id] = true;
        }
    });
    let from_id = variants
        .iter()
        .all(|(_, variant)| matches!(variant.fields, Fields::Unit))
        .then(|| {
            let arms = variants.iter().map(|(id, variant)| {
                let name = &variant.ident;
                quote!(n if n == #id as usize => Some(Self::#name),)
            });
            quote! {
                #[allow(clippy::unnecessary_cast)]
                pub const fn from_id(id: usize) -> Option<Self> {
                    match id { #(#arms)* _ => None }
                }
            }
        });
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        // Every registry entry needs a payload definition before it can be decoded.
        const _: [(); #registry::COUNT] = [(); #count];
        #[allow(clippy::unnecessary_cast)]
        const _: () = {
            let mut seen = [false; #registry::COUNT];
            #(#checks)*
        };

        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(clippy::unnecessary_cast)]
            pub const fn id(&self) -> i32 {
                match self { #(#arms)* }
            }
            #from_id
        }
    })
}

fn pair_variants_with_discriminants(
    attrs: &[Attribute],
    variants: impl IntoIterator<Item = Variant>,
) -> Result<Vec<(TokenStream, Variant)>> {
    if let Some((registry, prefix)) = registry_config(attrs)? {
        return variants
            .into_iter()
            .map(|variant| {
                let options = variant_options(&variant.attrs)?;
                if options.tag.is_some() {
                    return Err(syn::Error::new_spanned(
                        &variant,
                        "registry-tagged enum variants cannot have explicit tags",
                    ));
                }
                let id = options.key.unwrap_or_else(|| {
                    format_ident!(
                        "{}{}",
                        prefix,
                        variant.ident.to_string().to_shouty_snake_case()
                    )
                });
                Ok((quote!(#registry::#id), variant))
            })
            .collect();
    }

    let mut discriminant = 0;
    variants
        .into_iter()
        .map(|v| {
            let options = variant_options(&v.attrs)?;
            if let Some(key) = options.key {
                return Err(syn::Error::new_spanned(
                    key,
                    "registry key requires #[packet(registry = path)]",
                ));
            }
            if let Some(id) = options.tag {
                discriminant = id;
            }
            let id = discriminant;
            discriminant += 1;
            Ok((quote!(#id), v))
        })
        .collect::<Result<_>>()
}

fn registry_config(attrs: &[Attribute]) -> Result<Option<(Path, String)>> {
    let mut registry = None;
    let mut prefix: Option<LitStr> = None;
    for attr in attrs {
        if attr.path().is_ident("packet") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("registry") {
                    if registry.is_some() {
                        return Err(meta.error("duplicate registry path"));
                    }
                    registry = Some(meta.value()?.parse::<Path>()?);
                    Ok(())
                } else if meta.path.is_ident("registry_prefix") {
                    if prefix.is_some() {
                        return Err(meta.error("duplicate registry prefix"));
                    }
                    prefix = Some(meta.value()?.parse()?);
                    Ok(())
                } else if ["id", "name", "side", "state"]
                    .iter()
                    .any(|name| meta.path.is_ident(name))
                {
                    // These enum-level arguments belong to the Packet derive.
                    // Encode and Decode accept them without affecting field codecs.
                    meta.value()?.parse::<syn::Expr>()?;
                    Ok(())
                } else {
                    Err(meta.error("unrecognized packet argument"))
                }
            })?;
        }
    }
    if let Some(prefix) = &prefix {
        if registry.is_none() {
            return Err(syn::Error::new_spanned(
                prefix,
                "registry prefix requires #[packet(registry = path)]",
            ));
        }
        if !prefix.value().is_empty() {
            syn::parse_str::<syn::Ident>(&prefix.value())?;
        }
    }
    Ok(registry.map(|path| {
        (
            path,
            prefix.map_or_else(String::new, |prefix| prefix.value()),
        )
    }))
}

#[derive(Default)]
struct VariantOptions {
    tag: Option<i32>,
    key: Option<syn::Ident>,
    with: Option<Path>,
}

fn variant_options(attrs: &[Attribute]) -> Result<VariantOptions> {
    let mut options = VariantOptions::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("packet")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                if options.tag.is_some() {
                    return Err(meta.error("duplicate tag"));
                }
                options.tag = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            } else if meta.path.is_ident("key") {
                if options.key.is_some() {
                    return Err(meta.error("duplicate registry key"));
                }
                options.key = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("with") {
                if options.with.is_some() {
                    return Err(meta.error("duplicate payload codec"));
                }
                options.with = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error("unrecognized argument"));
            }
            Ok(())
        })?;
    }
    Ok(options)
}

fn field_codec(attrs: &[Attribute]) -> Result<Option<Path>> {
    let mut codec = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("packet")) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("with") {
                return Err(meta.error("expected `with = path` on a field"));
            }
            if codec.is_some() {
                return Err(meta.error("duplicate field codec"));
            }
            codec = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }
    Ok(codec)
}

fn check_variant_field_codecs(variant: &Variant) -> Result<()> {
    for field in &variant.fields {
        if field_codec(&field.attrs)?.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "a field codec cannot be combined with a variant payload codec",
            ));
        }
    }
    Ok(())
}

// Payload codecs receive/return a tuple of fields in declaration order.
fn variant_bindings(variant: &Variant) -> (TokenStream, Vec<syn::Ident>) {
    let name = &variant.ident;
    let fields: Vec<_> = variant
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .ident
                .clone()
                .unwrap_or_else(|| format_ident!("_field_{index}"))
        })
        .collect();
    let pattern = match &variant.fields {
        Fields::Named(_) => quote!(Self::#name { #(#fields,)* }),
        Fields::Unnamed(_) => quote!(Self::#name(#(#fields,)*)),
        Fields::Unit => quote!(Self::#name),
    };
    (pattern, fields)
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
