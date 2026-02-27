use core::fmt;
use std::borrow::Cow;
use std::fmt::Debug;
use std::{any::type_name, io::Write, marker::PhantomData};

use serde::de::{self, Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use valence_generated::{
    block::{BlockEntityKind, BlockKind},
    item::ItemKind,
};
use valence_ident::{ident, Ident};

use crate::{Decode, Encode, VarInt};

/// Trait implemented by items that can be indexed by a [`RegistryId`]
pub trait RegistryItem: Clone + Debug + PartialEq + Sized + Send + Sync + 'static {
    /// The resource location key for this registry (e.g. "minecraft:worldgen/biome").
    const KEY: Ident<&'static str>;
}

/// Trait implemented by items managed in dynamic registries that can be represented as NBT
pub trait DynamicRegistryItem: RegistryItem + Serialize + for<'de> Deserialize<'de> {}

/// A generic wrapper for a Registry ID.
///
/// `T` represents the type of registry this ID belongs to (e.g., `BlockKind`, `DimensionType`).
/// The internal integer is private to ensure IDs are only created from valid sources
/// (like decoding from a packet or converting from a static Registry enum).
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RegistryId<T: RegistryItem>(i32, PhantomData<T>);

impl<T: RegistryItem> Clone for RegistryId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
impl<T: RegistryItem> Copy for RegistryId<T> {}

impl<T: RegistryItem> fmt::Debug for RegistryId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(
            format!(
                "RegistryId<{}>",
                type_name::<T>().split("::").last().unwrap_or("?")
            )
            .as_str(),
        )
        .field(&self.0)
        .finish()
    }
}

// Ser/De as a string resource identifier.

// We only implement Serde for types that have a Static mapping.
// Dynamic registries (that rely on server data) cannot be serialized
// statelessly via Serde.
impl<T: StaticRegistry> Serialize for RegistryId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = T::from_registry_id(*self).ok_or_else(|| {
            serde::ser::Error::custom(format!(
                "ID {} is not valid for registry {}",
                self.0,
                T::KEY
            ))
        })?;

        let key = val.to_reg_key();
        serializer.serialize_str(key.as_str())
    }
}

impl<'de, T: StaticRegistry> Deserialize<'de> for RegistryId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RegistryVisitor<T>(PhantomData<T>);

        impl<'de, T: StaticRegistry> Visitor<'de> for RegistryVisitor<T> {
            type Value = RegistryId<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a namespaced registry key string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let ident_str = Ident::new(v).map_err(de::Error::custom)?;

                match T::from_reg_key(ident_str) {
                    Some(item) => Ok(item.to_registry_id()),
                    None => Err(E::custom(format!(
                        "Unknown registry key '{}' for registry {}",
                        v,
                        T::KEY
                    ))),
                }
            }
        }

        deserializer.deserialize_str(RegistryVisitor(PhantomData))
    }
}

impl<T: RegistryItem> RegistryId<T> {
    /// Creates a new RegistryId from a raw integer. In general you shouldnt use this.
    /// Users should obtain IDs by using the Registry Resource.
    pub const fn new(val: i32) -> Self {
        Self(val, PhantomData)
    }

    /// Returns the underlying raw integer ID.
    pub const fn get(&self) -> i32 {
        self.0
    }
}

impl<T: RegistryItem> Encode for RegistryId<T> {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        VarInt(self.0).encode(w)
    }
}

impl<'a, T: RegistryItem> Decode<'a> for RegistryId<T> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let val = VarInt::decode(r)?.0;
        // Trusted source: The protocol (network) provides this ID.
        Ok(Self::new(val))
    }
}

// Static registry implementors can be encoded and decoded statelessly.
pub trait StaticRegistry: RegistryItem {
    fn from_registry_id(id: RegistryId<Self>) -> Option<Self>
    where
        Self: Sized;
    fn to_registry_id(self) -> RegistryId<Self>;
    fn from_reg_key<'a>(name: impl Into<Ident<Cow<'a, str>>>) -> Option<Self>
    where
        Self: Sized;
    fn to_reg_key(self) -> Ident<&'static str>;
}

impl RegistryItem for BlockKind {
    const KEY: Ident<&'static str> = ident!("minecraft:block");
}

impl StaticRegistry for BlockKind {
    fn from_registry_id(id: RegistryId<Self>) -> Option<Self> {
        BlockKind::from_raw(id.get() as u16)
    }

    fn to_registry_id(self) -> RegistryId<Self> {
        RegistryId::new(self.to_raw() as i32)
    }

    fn from_reg_key<'a>(name: impl Into<Ident<Cow<'a, str>>>) -> Option<Self> {
        BlockKind::from_ident(name)
    }

    fn to_reg_key(self) -> Ident<&'static str> {
        self.ident()
    }
}

impl RegistryItem for BlockEntityKind {
    const KEY: Ident<&'static str> = ident!("minecraft:block_entity_type");
}

impl StaticRegistry for BlockEntityKind {
    fn from_registry_id(id: RegistryId<Self>) -> Option<Self> {
        BlockEntityKind::from_id(id.get() as u32)
    }

    fn to_registry_id(self) -> RegistryId<Self> {
        RegistryId::new(self.id() as i32)
    }

    fn from_reg_key<'a>(name: impl Into<Ident<Cow<'a, str>>>) -> Option<Self> {
        BlockEntityKind::from_ident(name)
    }

    fn to_reg_key(self) -> Ident<&'static str> {
        self.ident()
    }
}

impl RegistryItem for ItemKind {
    const KEY: Ident<&'static str> = ident!("minecraft:item");
}

impl StaticRegistry for ItemKind {
    fn from_registry_id(id: RegistryId<Self>) -> Option<Self> {
        ItemKind::from_raw(id.get() as u16)
    }

    fn to_registry_id(self) -> RegistryId<Self> {
        RegistryId::new(self.to_raw() as i32)
    }

    fn from_reg_key<'a>(name: impl Into<Ident<Cow<'a, str>>>) -> Option<Self> {
        ItemKind::from_ident(name)
    }

    fn to_reg_key(self) -> Ident<&'static str> {
        self.ident()
    }
}

// TODO: add every static registry here
