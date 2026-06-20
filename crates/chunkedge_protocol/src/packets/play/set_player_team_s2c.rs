use std::borrow::Cow;

use bitfield_struct::bitfield;
use chunkedge_binary::{Decode, Encode, TextComponent};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetPlayerTeamS2c<'a> {
    pub team_name: &'a str,
    pub mode: Mode<'a>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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
