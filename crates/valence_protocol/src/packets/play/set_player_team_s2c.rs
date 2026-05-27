use std::borrow::Cow;
use std::io::Write;

use anyhow::bail;
use bitfield_struct::bitfield;
use valence_binary::{Decode, Encode, TextComponent};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetPlayerTeamS2c<'a> {
    pub team_name: &'a str,
    pub mode: Mode<'a>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Mode<'a> {
    CreateTeam {
        team_display_name: Cow<'a, TextComponent>,
        friendly_flags: TeamFlags,
        name_tag_visibility: NameTagVisibility,
        collision_rule: CollisionRule,
        team_color: TeamColor,
        team_prefix: Cow<'a, TextComponent>,
        team_suffix: Cow<'a, TextComponent>,
        entities: Vec<&'a str>,
    },
    RemoveTeam,
    UpdateTeamInfo {
        team_display_name: Cow<'a, TextComponent>,
        friendly_flags: TeamFlags,
        name_tag_visibility: NameTagVisibility,
        collision_rule: CollisionRule,
        team_color: TeamColor,
        team_prefix: Cow<'a, TextComponent>,
        team_suffix: Cow<'a, TextComponent>,
    },
    AddEntities {
        entities: Vec<&'a str>,
    },
    RemoveEntities {
        entities: Vec<&'a str>,
    },
}

impl Encode for Mode<'_> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            Mode::CreateTeam {
                team_display_name,
                friendly_flags,
                name_tag_visibility,
                collision_rule,
                team_color,
                team_prefix,
                team_suffix,
                entities,
            } => {
                0_i8.encode(&mut w)?;
                team_display_name.encode(&mut w)?;
                friendly_flags.encode(&mut w)?;
                name_tag_visibility.encode(&mut w)?;
                collision_rule.encode(&mut w)?;
                team_color.encode(&mut w)?;
                team_prefix.encode(&mut w)?;
                team_suffix.encode(&mut w)?;
                entities.encode(&mut w)?;
            }
            Mode::RemoveTeam => 1_i8.encode(&mut w)?,
            Mode::UpdateTeamInfo {
                team_display_name,
                friendly_flags,
                name_tag_visibility,
                collision_rule,
                team_color,
                team_prefix,
                team_suffix,
            } => {
                2_i8.encode(&mut w)?;
                team_display_name.encode(&mut w)?;
                friendly_flags.encode(&mut w)?;
                name_tag_visibility.encode(&mut w)?;
                collision_rule.encode(&mut w)?;
                team_color.encode(&mut w)?;
                team_prefix.encode(&mut w)?;
                team_suffix.encode(&mut w)?;
            }
            Mode::AddEntities { entities } => {
                3_i8.encode(&mut w)?;
                entities.encode(&mut w)?;
            }
            Mode::RemoveEntities { entities } => {
                4_i8.encode(&mut w)?;
                entities.encode(&mut w)?;
            }
        }
        Ok(())
    }
}

impl<'a> Decode<'a> for Mode<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        Ok(match i8::decode(r)? {
            0 => Self::CreateTeam {
                team_display_name: Decode::decode(r)?,
                friendly_flags: Decode::decode(r)?,
                name_tag_visibility: Decode::decode(r)?,
                collision_rule: Decode::decode(r)?,
                team_color: Decode::decode(r)?,
                team_prefix: Decode::decode(r)?,
                team_suffix: Decode::decode(r)?,
                entities: Decode::decode(r)?,
            },
            1 => Self::RemoveTeam,
            2 => Self::UpdateTeamInfo {
                team_display_name: Decode::decode(r)?,
                friendly_flags: Decode::decode(r)?,
                name_tag_visibility: Decode::decode(r)?,
                collision_rule: Decode::decode(r)?,
                team_color: Decode::decode(r)?,
                team_prefix: Decode::decode(r)?,
                team_suffix: Decode::decode(r)?,
            },
            3 => Self::AddEntities {
                entities: Decode::decode(r)?,
            },
            4 => Self::RemoveEntities {
                entities: Decode::decode(r)?,
            },
            n => bail!("unknown update teams action of {n}"),
        })
    }
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct TeamFlags {
    pub friendly_fire: bool,
    pub see_invisible_teammates: bool,
    #[bits(6)]
    _pad: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum NameTagVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeams,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum CollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum TeamColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    Purple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    BrightGreen,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underlined,
    Italic,
    Reset,
}
