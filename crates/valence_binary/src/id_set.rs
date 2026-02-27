use core::fmt;
use serde::de::Error as DeErr;
use serde::ser::Error as SerErr;
use std::{any::type_name, io::Write, marker::PhantomData};

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use valence_ident::Ident;

use crate::{
    registry_id::{RegistryId, RegistryItem, StaticRegistry},
    Decode, Encode, VarInt,
};

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents a set of IDs in a certain registry, either directly (enumerated
/// IDs) or indirectly (tag name).
///
/// # Variants
///
/// - `NamedSet(String)`: Represents a named set of IDs defined by a tag.
/// - `AdHocSet(Vec<RegistryId>)`: Represents an ad-hoc set of IDs enumerated
///   inline.
///
/// # Serilized as:
///
/// - A string `"#{ident}"` for a named tag set. `NamedSet("{ident}")`
/// - A string `"{ident}"` for a single static registry id. `AdHocSet(vec![T::from_reg_key("{ident}")])`
/// - A list `["{ident}", "{ident}", ..]` for a inline set of ids. `AdHocSet(vec![T::from_reg_key("{ident}"), ..])`
pub enum IDSet<T: RegistryItem> {
    NamedSet(String),
    AdHocSet(Vec<RegistryId<T>>),
}

impl<T: StaticRegistry> Serialize for IDSet<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            IDSet::NamedSet(name) => serializer.serialize_str(&format!("#{}", name)),
            IDSet::AdHocSet(ids) => {
                if ids.len() == 1 {
                    if let Some(item) = T::from_registry_id(ids[0]) {
                        item.to_reg_key().serialize(serializer)
                    } else {
                        return Err(S::Error::custom(format!(
                            "invalid ID {} for {}",
                            ids[0].get(),
                            type_name::<T>()
                        )));
                    }
                } else {
                    let items: Vec<_> = ids
                        .iter()
                        .map(|id| {
                            if let Some(item) = T::from_registry_id(*id) {
                                Ok(item.to_reg_key())
                            } else {
                                Err(S::Error::custom(format!(
                                    "invalid ID {} for {}",
                                    id.get(),
                                    type_name::<T>()
                                )))
                            }
                        })
                        .collect::<Result<Vec<_>, S::Error>>()?;

                    items.serialize(serializer)
                }
            }
        }
    }
}

impl<'de, T: StaticRegistry> Deserialize<'de> for IDSet<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IDSetVisitor<T>(PhantomData<T>);

        impl<'de, T: StaticRegistry> Visitor<'de> for IDSetVisitor<T> {
            type Value = IDSet<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string starting with #, a registry ID string, or a list of registry IDs",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if let Some(tag_name) = v.strip_prefix('#') {
                    Ok(IDSet::NamedSet(tag_name.to_string()))
                } else {
                    Ok(IDSet::AdHocSet(vec![if let Some(item) =
                        T::from_reg_key(Ident::new(v).map_err(E::custom)?.as_str_ident())
                    {
                        item.to_registry_id()
                    } else {
                        return Err(E::custom(format!(
                            "invalid ident {} for {}",
                            v,
                            type_name::<T>()
                        )));
                    }]))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut ids = Vec::new();

                while let Some(key_str) = seq.next_element::<String>()? {
                    let ident = Ident::new(&key_str).map_err(de::Error::custom)?;

                    let item = T::from_reg_key(ident.as_str_ident()).ok_or_else(|| {
                        A::Error::custom(format!("Unknown registry key: {}", key_str))
                    })?;

                    ids.push(item.to_registry_id());
                }

                Ok(IDSet::AdHocSet(ids))
            }
        }

        deserializer.deserialize_any(IDSetVisitor(PhantomData))
    }
}

impl<T: RegistryItem + Encode> Encode for IDSet<T> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            IDSet::NamedSet(tag_name) => {
                VarInt(0).encode(&mut w)?;
                tag_name.encode(w)
            }
            IDSet::AdHocSet(ids) => {
                VarInt((ids.len() + 1) as i32).encode(&mut w)?;
                for id in ids {
                    id.encode(&mut w)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a, T: RegistryItem + Decode<'a>> Decode<'a> for IDSet<T> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let type_id = VarInt::decode(r)?.0;
        if type_id == 0 {
            let tag_name = String::decode(r)?;
            Ok(IDSet::NamedSet(tag_name))
        } else {
            let mut ids = Vec::with_capacity((type_id - 1) as usize);
            for _ in 0..(type_id - 1) {
                ids.push(RegistryId::new(VarInt::decode(r)?.0));
            }
            Ok(IDSet::AdHocSet(ids))
        }
    }
}
