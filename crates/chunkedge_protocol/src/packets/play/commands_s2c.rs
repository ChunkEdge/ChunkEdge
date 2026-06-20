use std::borrow::Cow;
use std::io::Write;

use anyhow::bail;
use byteorder::WriteBytesExt;
use valence_binary::{Decode, Encode, VarInt};
use valence_ident::Ident;

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Suggestion {
    AskServer,
    AllRecipes,
    AvailableSounds,
    SummonableEntities,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Parser<'a> {
    Bool,
    Float { min: Option<f32>, max: Option<f32> },
    Double { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i32>, max: Option<i32> },
    Long { min: Option<i64>, max: Option<i64> },
    String(StringArg),
    Entity { single: bool, only_players: bool },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
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
    ScoreHolder { allow_multiple: bool },
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
    GameMode,
    Time { min: i32 },
    ResourceOrTag { registry: Ident<Cow<'a, str>> },
    ResourceOrTagKey { registry: Ident<Cow<'a, str>> },
    Resource { registry: Ident<Cow<'a, str>> },
    ResourceKey { registry: Ident<Cow<'a, str>> },
    ResourceSelector { registry: Ident<Cow<'a, str>> },
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
                        Suggestion::AllRecipes => "minecraft:all_recipes",
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
                        "minecraft:all_recipes" => Suggestion::AllRecipes,
                        "minecraft:available_sounds" => Suggestion::AvailableSounds,
                        "minecraft:summonable_entities" => Suggestion::SummonableEntities,
                        other => bail!("unknown command suggestion type of \"{other}\""),
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

impl Encode for Parser<'_> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            Parser::Bool => 0_u8.encode(&mut w)?,
            Parser::Float { min, max } => {
                1_u8.encode(&mut w)?;

                (u8::from(min.is_some()) | (u8::from(max.is_some()) * 0x2)).encode(&mut w)?;

                if let Some(min) = min {
                    min.encode(&mut w)?;
                }

                if let Some(max) = max {
                    max.encode(&mut w)?;
                }
            }
            Parser::Double { min, max } => {
                2_u8.encode(&mut w)?;

                (u8::from(min.is_some()) | (u8::from(max.is_some()) * 0x2)).encode(&mut w)?;

                if let Some(min) = min {
                    min.encode(&mut w)?;
                }

                if let Some(max) = max {
                    max.encode(&mut w)?;
                }
            }
            Parser::Integer { min, max } => {
                3_u8.encode(&mut w)?;

                (u8::from(min.is_some()) | (u8::from(max.is_some()) * 0x2)).encode(&mut w)?;

                if let Some(min) = min {
                    min.encode(&mut w)?;
                }

                if let Some(max) = max {
                    max.encode(&mut w)?;
                }
            }
            Parser::Long { min, max } => {
                4_u8.encode(&mut w)?;

                (u8::from(min.is_some()) | (u8::from(max.is_some()) * 0x2)).encode(&mut w)?;

                if let Some(min) = min {
                    min.encode(&mut w)?;
                }

                if let Some(max) = max {
                    max.encode(&mut w)?;
                }
            }
            Parser::String(arg) => {
                5_u8.encode(&mut w)?;
                arg.encode(&mut w)?;
            }
            Parser::Entity {
                single,
                only_players,
            } => {
                6_u8.encode(&mut w)?;
                (u8::from(*single) | (u8::from(*only_players) * 0x2)).encode(&mut w)?;
            }
            Parser::GameProfile => 7_u8.encode(&mut w)?,
            Parser::BlockPos => 8_u8.encode(&mut w)?,
            Parser::ColumnPos => 9_u8.encode(&mut w)?,
            Parser::Vec3 => 10_u8.encode(&mut w)?,
            Parser::Vec2 => 11_u8.encode(&mut w)?,
            Parser::BlockState => 12_u8.encode(&mut w)?,
            Parser::BlockPredicate => 13_u8.encode(&mut w)?,
            Parser::ItemStack => 14_u8.encode(&mut w)?,
            Parser::ItemPredicate => 15_u8.encode(&mut w)?,
            Parser::Color => 16_u8.encode(&mut w)?,
            Parser::Component => 17_u8.encode(&mut w)?,
            Parser::Style => 18_u8.encode(&mut w)?,
            Parser::Message => 19_u8.encode(&mut w)?,
            Parser::NbtCompoundTag => 20_u8.encode(&mut w)?,
            Parser::NbtTag => 21_u8.encode(&mut w)?,
            Parser::NbtPath => 22_u8.encode(&mut w)?,
            Parser::Objective => 23_u8.encode(&mut w)?,
            Parser::ObjectiveCriteria => 24_u8.encode(&mut w)?,
            Parser::Operation => 25_u8.encode(&mut w)?,
            Parser::Particle => 26_u8.encode(&mut w)?,
            Parser::Angle => 27_u8.encode(&mut w)?,
            Parser::Rotation => 28_u8.encode(&mut w)?,
            Parser::ScoreboardSlot => 29_u8.encode(&mut w)?,
            Parser::ScoreHolder { allow_multiple } => {
                30_u8.encode(&mut w)?;
                allow_multiple.encode(&mut w)?;
            }
            Parser::Swizzle => 31_u8.encode(&mut w)?,
            Parser::Team => 32_u8.encode(&mut w)?,
            Parser::ItemSlot => 33_u8.encode(&mut w)?,
            Parser::ItemSlots => 34_u8.encode(&mut w)?,
            Parser::ResourceLocation => 35_u8.encode(&mut w)?,
            Parser::Function => 36_u8.encode(&mut w)?,
            Parser::EntityAnchor => 37_u8.encode(&mut w)?,
            Parser::IntRange => 38_u8.encode(&mut w)?,
            Parser::FloatRange => 39_u8.encode(&mut w)?,
            Parser::Dimension => 40_u8.encode(&mut w)?,
            Parser::GameMode => 41_u8.encode(&mut w)?,
            Parser::Time { min } => {
                42_u8.encode(&mut w)?;
                min.encode(&mut w)?;
            }
            Parser::ResourceOrTag { registry } => {
                43_u8.encode(&mut w)?;
                registry.encode(&mut w)?;
            }
            Parser::ResourceOrTagKey { registry } => {
                44_u8.encode(&mut w)?;
                registry.encode(&mut w)?;
            }
            Parser::Resource { registry } => {
                45_u8.encode(&mut w)?;
                registry.encode(&mut w)?;
            }
            Parser::ResourceKey { registry } => {
                46_u8.encode(&mut w)?;
                registry.encode(&mut w)?;
            }
            Parser::ResourceSelector { registry } => {
                47_u8.encode(&mut w)?;
                registry.encode(&mut w)?;
            }
            Parser::TemplateMirror => 48_u8.encode(&mut w)?,
            Parser::TemplateRotation => 49_u8.encode(&mut w)?,
            Parser::Heightmap => 50_u8.encode(&mut w)?,
            Parser::LootTable => 51_u8.encode(&mut w)?,
            Parser::LootPredicate => 52_u8.encode(&mut w)?,
            Parser::LootModifier => 53_u8.encode(&mut w)?,
            Parser::Dialog => 55_u8.encode(&mut w)?,
            Parser::Uuid => 56_u8.encode(&mut w)?,
        }

        Ok(())
    }
}

impl<'a> Decode<'a> for Parser<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        fn decode_min_max<'a, T: Decode<'a>>(
            r: &mut &'a [u8],
        ) -> anyhow::Result<(Option<T>, Option<T>)> {
            let flags = u8::decode(r)?;

            let min = if flags & 0x1 != 0 {
                Some(T::decode(r)?)
            } else {
                None
            };

            let max = if flags & 0x2 != 0 {
                Some(T::decode(r)?)
            } else {
                None
            };

            Ok((min, max))
        }

        Ok(match u8::decode(r)? {
            0 => Self::Bool,
            1 => {
                let (min, max) = decode_min_max(r)?;
                Self::Float { min, max }
            }
            2 => {
                let (min, max) = decode_min_max(r)?;
                Self::Double { min, max }
            }
            3 => {
                let (min, max) = decode_min_max(r)?;
                Self::Integer { min, max }
            }
            4 => {
                let (min, max) = decode_min_max(r)?;
                Self::Long { min, max }
            }
            5 => Self::String(StringArg::decode(r)?),
            6 => {
                let flags = u8::decode(r)?;
                Self::Entity {
                    single: flags & 0x1 != 0,
                    only_players: flags & 0x2 != 0,
                }
            }
            7 => Self::GameProfile,
            8 => Self::BlockPos,
            9 => Self::ColumnPos,
            10 => Self::Vec3,
            11 => Self::Vec2,
            12 => Self::BlockState,
            13 => Self::BlockPredicate,
            14 => Self::ItemStack,
            15 => Self::ItemPredicate,
            16 => Self::Color,
            17 => Self::Component,
            18 => Self::Style,
            19 => Self::Message,
            20 => Self::NbtCompoundTag,
            21 => Self::NbtTag,
            22 => Self::NbtPath,
            23 => Self::Objective,
            24 => Self::ObjectiveCriteria,
            25 => Self::Operation,
            26 => Self::Particle,
            27 => Self::Angle,
            28 => Self::Rotation,
            29 => Self::ScoreboardSlot,
            30 => Self::ScoreHolder {
                allow_multiple: bool::decode(r)?,
            },
            31 => Self::Swizzle,
            32 => Self::Team,
            33 => Self::ItemSlot,
            34 => Self::ItemSlots,
            35 => Self::ResourceLocation,
            36 => Self::Function,
            37 => Self::EntityAnchor,
            38 => Self::IntRange,
            39 => Self::FloatRange,
            40 => Self::Dimension,
            41 => Self::GameMode,
            42 => Self::Time {
                min: i32::decode(r)?,
            },
            43 => Self::ResourceOrTag {
                registry: Ident::decode(r)?,
            },
            44 => Self::ResourceOrTagKey {
                registry: Ident::decode(r)?,
            },
            45 => Self::Resource {
                registry: Ident::decode(r)?,
            },
            46 => Self::ResourceKey {
                registry: Ident::decode(r)?,
            },
            47 => Self::ResourceSelector {
                registry: Ident::decode(r)?,
            },
            48 => Self::TemplateMirror,
            49 => Self::TemplateRotation,
            50 => Self::Heightmap,
            51 => Self::LootTable,
            52 => Self::LootPredicate,
            53 => Self::LootModifier,
            55 => Self::Dialog,
            56 => Self::Uuid,
            n => bail!("unknown command parser ID of {n}"),
        })
    }
}
