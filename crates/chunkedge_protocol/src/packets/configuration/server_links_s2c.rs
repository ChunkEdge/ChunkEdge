use std::borrow::Cow;

use chunkedge_binary::{Decode, Either, Encode, TextComponent};

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct ServerLinksS2c<'a> {
    pub links: Vec<ServerLink<'a>>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ServerLink<'a> {
    pub label: Either<BuiltInLinkType, TextComponent>,
    pub url: Cow<'a, str>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Encode, Decode)]
pub enum BuiltInLinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}
