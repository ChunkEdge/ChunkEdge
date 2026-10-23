use std::borrow::Cow;
use std::io::Write;

use anyhow::bail;
use byteorder::WriteBytesExt;
use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_generated::command_argument_type as parser_id;
use chunkedge_ident::Ident;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct CommandsS2c<'a> {
    pub commands: Vec<Node<'a>>,
    pub root_index: VarInt,
}

#[derive(Clone, Debug)]
pub struct Node<'a> {
    pub data: NodeData<'a>,
    pub executable: bool,
    pub children: Vec<VarInt>,
    pub redirect_node: Option<VarInt>,
    /// Set if the node requires the player to have a permission level above 0.
    pub is_restricted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeData<'a> {
    Root,
    Literal {
        name: Cow<'a, str>,
    },
    Argument {
        name: Cow<'a, str>,
        parser: Parser<'a>,
        suggestion: Option<Suggestion>,
    },
}

/// Vanilla `SuggestionProviders`. The client treats an unknown provider as
/// [`Suggestion::AskServer`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Suggestion {
    AskServer,
    AvailableSounds,
    SummonableEntities,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, chunkedge_protocol_macros::RegistryId)]
#[packet(registry = parser_id, registry_prefix = "MINECRAFT_")]
pub enum Parser<'a> {
    #[packet(key = BRIGADIER_BOOL)]
    Bool,
    #[packet(key = BRIGADIER_FLOAT, with = bounds)]
    Float {
        min: Option<f32>,
        max: Option<f32>,
    },
    #[packet(key = BRIGADIER_DOUBLE, with = bounds)]
    Double {
        min: Option<f64>,
        max: Option<f64>,
    },
    #[packet(key = BRIGADIER_INTEGER, with = bounds)]
    Integer {
        min: Option<i32>,
        max: Option<i32>,
    },
    #[packet(key = BRIGADIER_LONG, with = bounds)]
    Long {
        min: Option<i64>,
        max: Option<i64>,
    },
    #[packet(key = BRIGADIER_STRING)]
    String(StringArg),
    #[packet(with = entity_flags)]
    Entity {
        single: bool,
        only_players: bool,
    },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    #[packet(key = MINECRAFT_TEAM_COLOR)]
    Color,
    HexColor,
    Component,
    Style,
    Message,
    NbtCompoundTag,
    NbtTag,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Angle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder {
        allow_multiple: bool,
    },
    Swizzle,
    Team,
    ItemSlot,
    ItemSlots,
    ResourceLocation,
    Function,
    EntityAnchor,
    IntRange,
    FloatRange,
    Dimension,
    #[packet(key = MINECRAFT_GAMEMODE)]
    GameMode,
    Time {
        min: i32,
    },
    ResourceOrTag {
        registry: Ident<Cow<'a, str>>,
    },
    ResourceOrTagKey {
        registry: Ident<Cow<'a, str>>,
    },
    Resource {
        registry: Ident<Cow<'a, str>>,
    },
    ResourceKey {
        registry: Ident<Cow<'a, str>>,
    },
    ResourceSelector {
        registry: Ident<Cow<'a, str>>,
    },
    TemplateMirror,
    TemplateRotation,
    Heightmap,
    LootTable,
    LootPredicate,
    LootModifier,
    Dialog,
    Uuid,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum StringArg {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

impl Encode for Node<'_> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        let node_type = match &self.data {
            NodeData::Root => 0,
            NodeData::Literal { .. } => 1,
            NodeData::Argument { .. } => 2,
        };

        let has_suggestion = matches!(
            &self.data,
            NodeData::Argument {
                suggestion: Some(_),
                ..
            }
        );

        let flags: u8 = node_type
            | (u8::from(self.executable) * 0x04)
            | (u8::from(self.redirect_node.is_some()) * 0x08)
            | (u8::from(has_suggestion) * 0x10)
            | (u8::from(self.is_restricted) * 0x20);

        w.write_u8(flags)?;

        self.children.encode(&mut w)?;

        if let Some(redirect_node) = self.redirect_node {
            redirect_node.encode(&mut w)?;
        }

        match &self.data {
            NodeData::Root => {}
            NodeData::Literal { name } => {
                name.encode(&mut w)?;
            }
            NodeData::Argument {
                name,
                parser,
                suggestion,
            } => {
                name.encode(&mut w)?;
                parser.encode(&mut w)?;

                if let Some(suggestion) = suggestion {
                    match suggestion {
                        Suggestion::AskServer => "minecraft:ask_server",
                        Suggestion::AvailableSounds => "minecraft:available_sounds",
                        Suggestion::SummonableEntities => "minecraft:summonable_entities",
                    }
                    .encode(&mut w)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Decode<'a> for Node<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let flags = u8::decode(r)?;

        let children = Vec::decode(r)?;

        let redirect_node = if flags & 0x08 != 0 {
            Some(VarInt::decode(r)?)
        } else {
            None
        };

        let node_data = match flags & 0x3 {
            0 => NodeData::Root,
            1 => NodeData::Literal {
                name: <Cow<'a, str>>::decode(r)?,
            },
            2 => NodeData::Argument {
                name: <Cow<'a, str>>::decode(r)?,
                parser: Parser::decode(r)?,
                suggestion: if flags & 0x10 != 0 {
                    Some(match Ident::<Cow<str>>::decode(r)?.as_str() {
                        "minecraft:ask_server" => Suggestion::AskServer,
                        "minecraft:available_sounds" => Suggestion::AvailableSounds,
                        "minecraft:summonable_entities" => Suggestion::SummonableEntities,
                        // Vanilla `SuggestionProviders.getProvider` falls back to
                        // asking the server for unknown names.
                        _ => Suggestion::AskServer,
                    })
                } else {
                    None
                },
            },
            n => bail!("invalid node type of {n}"),
        };

        Ok(Self {
            children,
            data: node_data,
            executable: flags & 0x04 != 0,
            redirect_node,
            is_restricted: flags & 0x20 != 0,
        })
    }
}

// These payloads pack presence/boolean flags rather than encoding fields sequentially.
mod bounds {
    use super::*;

    pub(super) fn encode<T: Encode>(
        (min, max): (&Option<T>, &Option<T>),
        mut w: impl Write,
    ) -> anyhow::Result<()> {
        (u8::from(min.is_some()) | (u8::from(max.is_some()) << 1)).encode(&mut w)?;
        if let Some(min) = min {
            min.encode(&mut w)?;
        }
        if let Some(max) = max {
            max.encode(&mut w)?;
        }
        Ok(())
    }

    pub(super) fn decode<'a, T: Decode<'a>>(
        r: &mut &'a [u8],
    ) -> anyhow::Result<(Option<T>, Option<T>)> {
        let flags = u8::decode(r)?;
        let min = if flags & 1 != 0 {
            Some(T::decode(r)?)
        } else {
            None
        };
        let max = if flags & 2 != 0 {
            Some(T::decode(r)?)
        } else {
            None
        };
        Ok((min, max))
    }
}

mod entity_flags {
    use super::*;

    pub(super) fn encode(
        (single, only_players): (&bool, &bool),
        mut w: impl Write,
    ) -> anyhow::Result<()> {
        (u8::from(*single) | (u8::from(*only_players) << 1)).encode(&mut w)
    }

    pub(super) fn decode(r: &mut &[u8]) -> anyhow::Result<(bool, bool)> {
        let flags = u8::decode(r)?;
        Ok((flags & 1 != 0, flags & 2 != 0))
    }
}
