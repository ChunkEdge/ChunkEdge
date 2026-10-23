use std::collections::BTreeMap;

use chunkedge_build_utils::rerun_if_changed;
use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/command_argument_types.json"]);

    let types: BTreeMap<String, i32> =
        serde_json::from_str(include_str!("../extracted/command_argument_types.json"))?;
    let count = types.len();
    let constants = types.into_iter().map(|(name, id)| {
        let ident = format_ident!("{}", name.replace(':', "_").to_shouty_snake_case());
        let doc = format!("`{name}`");
        quote! {
            #[doc = #doc]
            pub const #ident: i32 = #id;
        }
    });

    Ok(quote! {
        pub const COUNT: usize = #count;
        #(#constants)*
    })
}
