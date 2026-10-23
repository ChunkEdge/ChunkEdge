use std::collections::BTreeMap;

use chunkedge_build_utils::rerun_if_changed;
use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/item_component_types.json"]);
    let types: BTreeMap<String, usize> =
        serde_json::from_str(include_str!("../extracted/item_component_types.json"))?;
    let count = types.len();
    let mut ids: Vec<_> = types.values().copied().collect();
    ids.sort_unstable();
    anyhow::ensure!(
        ids.into_iter().eq(0..count),
        "item component IDs must be contiguous"
    );
    let constants = types.into_iter().map(|(name, id)| {
        let name = format_ident!("{}", name.to_shouty_snake_case());
        quote! { pub const #name: usize = #id; }
    });
    Ok(quote! {
        pub const COUNT: usize = #count;
        #(#constants)*
    })
}
