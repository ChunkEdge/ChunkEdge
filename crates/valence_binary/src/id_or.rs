use std::fmt::Debug;
use std::io::Write;

use crate::registry_id::{RegistryId, RegistryItem};
use anyhow::Error;

use crate::{Decode, Encode, VarInt};

#[derive(Clone, Debug, PartialEq)]
pub enum IdOr<T: RegistryItem> {
    Id(RegistryId<T>),
    Inline(T),
}

impl<T: RegistryItem> IdOr<T> {
    pub fn id<I: Into<RegistryId<T>>>(id: I) -> Self {
        Self::Id(id.into())
    }

    pub fn inline(value: T) -> Self {
        Self::Inline(value)
    }
}

impl<T: Encode + RegistryItem> Encode for IdOr<T> {
    fn encode(&self, mut buf: impl Write) -> anyhow::Result<()> {
        match self {
            Self::Id(id) => (id.get() + 1).encode(buf),
            Self::Inline(value) => {
                VarInt(0).encode(&mut buf).unwrap();
                value.encode(&mut buf)
            }
        }
    }
}

impl<'a, T: Decode<'a> + RegistryItem> Decode<'a> for IdOr<T> {
    fn decode(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let id = VarInt::decode(buf)?;
        if id == VarInt(0) {
            let value = T::decode(buf)?;
            Ok(Self::Inline(value))
        } else {
            let registry_id = RegistryId::new(id.0 - 1);
            Ok(Self::Id(registry_id))
        }
    }
}
