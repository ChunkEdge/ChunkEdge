// TODO: simplify this and bound packet fields where appropriate.

use std::borrow::Cow;
use std::io::Write;

use chunkedge_binary::{Decode, Encode, TextComponent, VarInt};
use chunkedge_generated::packet_id;
use chunkedge_ident::Ident;
use chunkedge_item::ItemStackTemplate;

use crate::Packet;

pub type UpdateAdvancementsS2c<'a> = GenericUpdateAdvancementsS2c<'a, PositionedAdvancement<'a>>;

#[derive(Clone, Debug, Encode, Decode)]
pub struct PositionedAdvancement<'a> {
    pub id: Ident<Cow<'a, str>>,
    pub advancement: Advancement<'a, ItemStackTemplate>,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(id = packet_id::PLAY_UPDATE_ADVANCEMENTS_S2C)]
pub struct GenericUpdateAdvancementsS2c<'a, AM: 'a> {
    pub reset: bool,
    pub advancement_mapping: Vec<AM>,
    pub identifiers: Vec<Ident<Cow<'a, str>>>,
    pub progress_mapping: Vec<(Ident<Cow<'a, str>>, Vec<AdvancementCriteria<'a>>)>,
    pub show_advancements: bool,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct Advancement<'a, I> {
    pub parent_id: Option<Ident<Cow<'a, str>>>,
    pub display_data: Option<AdvancementDisplay<'a, I>>,
    pub requirements: Vec<AdvancementRequirements<'a>>,
    pub sends_telemetry_data: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct AdvancementRequirements<'a> {
    pub requirement: Vec<&'a str>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdvancementDisplay<'a, I> {
    pub title: Cow<'a, TextComponent>,
    pub description: Cow<'a, TextComponent>,
    /// Item icon encoded with [`ItemStackTemplate`].
    pub icon: I,
    pub frame_type: VarInt,
    pub flags: i32,
    pub background_texture: Option<Ident<Cow<'a, str>>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct AdvancementCriteria<'a> {
    pub criterion_identifier: Cow<'a, str>,
    /// If present, the criteria has been achieved at the
    /// time wrapped; time represented as millis since epoch
    pub criterion_progress: Option<i64>,
}

impl<I: Encode> Encode for AdvancementDisplay<'_, I> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.title.encode(&mut w)?;
        self.description.encode(&mut w)?;
        self.icon.encode(&mut w)?;
        self.frame_type.encode(&mut w)?;
        self.flags.encode(&mut w)?;

        match self.background_texture.as_ref() {
            None => {}
            Some(texture) => texture.encode(&mut w)?,
        }

        Ok(())
    }
}

impl<'a, I: Decode<'a>> Decode<'a> for AdvancementDisplay<'a, I> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let title = <Cow<'a, TextComponent>>::decode(r)?;
        let description = <Cow<'a, TextComponent>>::decode(r)?;
        let icon = I::decode(r)?;
        let frame_type = VarInt::decode(r)?;
        let flags = i32::decode(r)?;

        let background_texture = if flags & 1 == 1 {
            Some(Ident::decode(r)?)
        } else {
            None
        };

        Ok(Self {
            title,
            description,
            icon,
            frame_type,
            flags,
            background_texture,
        })
    }
}
