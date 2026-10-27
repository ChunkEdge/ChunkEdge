use std::collections::BTreeMap;

use chunkedge_build_utils::{ident, rerun_if_changed, write_generated_file};
use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

pub fn main() -> anyhow::Result<()> {
    write_generated_file(build()?, "translation_keys.rs")
}

fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/translation_keys.json"]);

    let translations =
        serde_json::from_str::<Vec<Translation>>(include_str!("extracted/translation_keys.json"))?;

    let mut seen = BTreeMap::new();

    let translation_key_consts = translations
        .iter()
        .map(|translation| {
            // Uppercase before splitting on separators so camel-case words stay
            // intact: book.editTitle -> BOOK_EDITTITLE, book.edit.title -> BOOK_EDIT_TITLE.
            let const_id = ident(translation.key.to_uppercase().to_shouty_snake_case());
            let key = &translation.key;
            if let Some(previous) = seen.insert(const_id.to_string(), key) {
                anyhow::ensure!(previous != key, "duplicate translation key {key:?}");
                anyhow::bail!(
                    "translation keys {previous:?} and {key:?} both generate constant {const_id}"
                );
            }
            let english_translation = &translation.english_translation;
            let doc = format!("\"{}\"", escape(english_translation)).replace('`', "\\`");

            Ok(quote! {
                #[doc = #doc]
                pub const #const_id: &str = #key;
            })
        })
        .collect::<anyhow::Result<Vec<TokenStream>>>()?;

    Ok(quote! {
        #(#translation_key_consts)*
    })
}

#[derive(Deserialize, Clone, Debug)]
struct Translation {
    key: String,
    english_translation: String,
}

/// Escapes characters that have special meaning inside docs.
fn escape(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}
