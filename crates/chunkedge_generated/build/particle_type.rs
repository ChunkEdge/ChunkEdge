use std::collections::BTreeMap;

use chunkedge_build_utils::rerun_if_changed;
use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct Misc {
    particle_type: BTreeMap<String, i32>,
}

pub(crate) fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/misc.json"]);

    let misc: Misc = serde_json::from_str(include_str!("../extracted/misc.json"))?;
    let count = misc.particle_type.len();
    let constants = misc.particle_type.into_iter().map(|(name, id)| {
        let ident = format_ident!("{}", name.to_shouty_snake_case());
        quote! {
            pub const #ident: i32 = #id;
        }
    });

    Ok(quote! {
        pub const COUNT: usize = #count;
        #(#constants)*
    })
}
