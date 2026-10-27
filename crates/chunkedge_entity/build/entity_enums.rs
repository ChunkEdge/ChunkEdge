use std::collections::{BTreeMap, BTreeSet};

use anyhow::Context;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Deserialize)]
struct ExtractedEntity {
    fields: Vec<ExtractedField>,
}

#[derive(Deserialize)]
struct ExtractedField {
    #[serde(rename = "type")]
    kind: String,
    default_value: JsonValue,
}

pub(super) fn build() -> anyhow::Result<TokenStream> {
    let misc: BTreeMap<String, BTreeMap<String, i32>> =
        serde_json::from_str(chunkedge_generated::extracted::MISC_JSON)?;
    let enum_keys: BTreeSet<_> = misc
        .keys()
        .filter(|name| enum_name(name).is_some())
        .cloned()
        .collect();
    let defaults = extracted_defaults(&enum_keys)?;
    let mut output = TokenStream::new();

    for (registry_name, entries) in &misc {
        let Some(rust_name) = enum_name(registry_name) else {
            continue;
        };
        let default_name = defaults
            .get(registry_name)
            .with_context(|| format!("missing default for {registry_name} in entities.json"))?;

        let mut sorted: Vec<_> = entries.iter().collect();
        sorted.sort_by_key(|(_, id)| **id);
        let mut names = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut variants = Vec::new();
        let mut found_default = false;

        for (name, id) in sorted {
            anyhow::ensure!(*id >= 0, "negative ID {id} in {registry_name}");
            anyhow::ensure!(ids.insert(*id), "duplicate ID {id} in {registry_name}");

            let local_name = name.strip_prefix("minecraft:").unwrap_or(name);
            let rust_variant = variant_name(registry_name, name);
            anyhow::ensure!(
                names.insert(rust_variant.clone()),
                "duplicate Rust variant {rust_variant} in {registry_name}"
            );

            let variant = format_ident!("{rust_variant}");
            let is_default = local_name == default_name;
            found_default |= is_default;
            let default = is_default.then(|| quote!(#[default]));
            variants.push(quote! {
                #default
                #[packet(tag = #id)]
                #variant = #id,
            });
        }

        anyhow::ensure!(
            found_default,
            "default {default_name} is absent from {registry_name} in misc.json"
        );

        let enum_name = format_ident!("{rust_name}");
        output.extend(quote! {
            #[repr(i32)]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Encode, Decode)]
            pub enum #enum_name {
                #(#variants)*
            }
        });
    }

    Ok(output)
}

fn enum_name(registry_name: &str) -> Option<String> {
    // Initially, these enums were hardcoded instead of generated as they are now.
    // The names below are kept for backwards compatibility with the existing public API.
    // We should probably break backwards compatibility at some point to simplify this and
    // avoid confusion, but for now we keep the old names.
    match registry_name {
        "villager_type" => Some("VillagerKind".to_owned()),
        "entity_pose" => Some("Pose".to_owned()),
        "villager_profession" | "humanoid_arm" => Some(registry_name.to_pascal_case()),
        "painting_variant" => None, // Generated with dimensions in painting_kind.rs.
        name if name.ends_with("_variant") => Some(format!(
            "{}Kind",
            name.trim_end_matches("_variant").to_pascal_case()
        )),
        name if name.ends_with("_state") => Some(name.to_pascal_case()),
        _ => None,
    }
}

fn extracted_defaults(enum_keys: &BTreeSet<String>) -> anyhow::Result<BTreeMap<String, String>> {
    let entities: BTreeMap<String, ExtractedEntity> =
        serde_json::from_str(include_str!("../extracted/entities.json"))?;
    let mut defaults = BTreeMap::new();

    for (entity_name, entity) in entities {
        for field in entity.fields {
            if field.kind == "villager_data" {
                for (registry_name, field_name) in [
                    ("villager_type", "type"),
                    ("villager_profession", "profession"),
                ] {
                    let value = field
                        .default_value
                        .get(field_name)
                        .and_then(JsonValue::as_str)
                        .with_context(|| {
                            format!("missing {field_name} in {entity_name} villager_data")
                        })?;
                    record_default(&mut defaults, registry_name, value)?;
                }
            } else if enum_keys.contains(&field.kind) {
                let value = field.default_value.as_str().with_context(|| {
                    format!("non-string {} default in {entity_name}", field.kind)
                })?;
                record_default(&mut defaults, &field.kind, value)?;
            }
        }
    }

    Ok(defaults)
}

fn record_default(
    defaults: &mut BTreeMap<String, String>,
    registry_name: &str,
    value: &str,
) -> anyhow::Result<()> {
    let local_name = value.strip_prefix("minecraft:").unwrap_or(value);
    if let Some(previous) = defaults.insert(registry_name.to_owned(), local_name.to_owned()) {
        anyhow::ensure!(
            previous == local_name,
            "conflicting defaults for {registry_name}: {previous} and {local_name}"
        );
    }
    Ok(())
}

pub(super) fn variant_name(registry_name: &str, name: &str) -> String {
    let local_name = name.strip_prefix("minecraft:").unwrap_or(name);
    // Keep the existing public Rust name while using Mojang's registry ID.
    if registry_name == "entity_pose" && local_name == "crouching" {
        "Sneaking".to_owned()
    } else {
        local_name.to_pascal_case()
    }
}
