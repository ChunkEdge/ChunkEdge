use std::borrow::Cow;
use std::io::Write;

use anyhow::{Context, bail, ensure};
use byteorder::{BigEndian, WriteBytesExt};
use chunkedge_nbt::binary::ToModifiedUtf8;
use chunkedge_nbt::{Tag, Value};
use chunkedge_text::{IntoText, Text};
use serde::Deserialize;
use serde::de::IntoDeserializer;

use crate::{Decode, Encode};

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)] // if you change this you have to remove the unsafe code!
pub struct TextComponent {
    pub text: Text,
}

impl TextComponent {
    /// Zero-copy cast from a `Cow<Text>` to a `Cow<TextComponent>`.
    ///
    /// # Safety
    /// This is safe because `TextComponent` is #[repr(transparent)] wrapper
    /// around Text.
    pub fn from_cow_text<'a>(cow: Cow<'a, Text>) -> Cow<'a, TextComponent> {
        match cow {
            Cow::Borrowed(b) => {
                // SAFETY: TextComponent has the exact same memory layout as Text.
                let ptr = b as *const Text as *const TextComponent;
                Cow::Borrowed(unsafe { &*ptr })
            }
            Cow::Owned(o) => Cow::Owned(TextComponent { text: o }),
        }
    }

    pub fn as_text(&self) -> &Text {
        &self.text
    }
}

impl<'a> IntoText<'a> for TextComponent {
    fn into_cow_text(self) -> Cow<'a, Text> {
        // Since we wrap Text, we just return it.
        Cow::Owned(self.text)
    }
}

pub trait IntoTextComponent<'a> {
    fn into_text_component(self) -> TextComponent;
    fn into_cow_text_component(self) -> Cow<'a, TextComponent>;
}

impl<'a, T: IntoText<'a>> IntoTextComponent<'a> for T {
    fn into_text_component(self) -> TextComponent {
        TextComponent {
            text: self.into_cow_text().into_owned(),
        }
    }

    fn into_cow_text_component(self) -> Cow<'a, TextComponent> {
        let cow = self.into_cow_text();
        TextComponent::from_cow_text(cow)
    }
}

impl From<Text> for TextComponent {
    fn from(value: Text) -> Self {
        TextComponent { text: value }
    }
}

impl Encode for TextComponent {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        if self.text.is_plain() {
            // Encode as NBT String
            let mut w = w;
            w.write_u8(Tag::String as u8)?;

            let chunkedge_text::TextContent::Text { text: string } = &self.text.content else {
                // is_plain should mean this is unreachable
                unreachable!()
            };

            let len = string.modified_uf8_len();

            match u16::try_from(len) {
                Ok(n) => w.write_u16::<BigEndian>(n)?,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "string of length {len} exceeds maximum of u16::MAX"
                    ));
                }
            }

            string.to_modified_utf8(len, &mut w)?;
            Ok(())
        } else {
            // Encode as Compound
            w.write_u8(Tag::Compound as u8)?;
            self.text.encode(&mut w)
        }
    }
}

impl Decode<'_> for TextComponent {
    fn decode(r: &mut &'_ [u8]) -> anyhow::Result<Self> {
        let value = chunkedge_nbt::from_unnamed_value(r).context("decoding text NBT")?;
        Ok(Self {
            text: text_from_nbt(value)?,
        })
    }
}

fn text_from_nbt(value: Value) -> anyhow::Result<Text> {
    match value {
        Value::String(text) => Ok(text.into()),
        Value::Compound(compound) => {
            Text::deserialize(compound.into_deserializer()).context("deserializing text NBT")
        }
        Value::List(list) => {
            ensure!(!list.is_empty(), "text component list must not be empty");
            let mut components = list.into_iter();
            let mut text = text_from_nbt(components.next().unwrap())?;
            for component in components {
                text += text_from_nbt(component)?;
            }
            Ok(text)
        }
        _ => bail!("expected a string, compound or list text component"),
    }
}
