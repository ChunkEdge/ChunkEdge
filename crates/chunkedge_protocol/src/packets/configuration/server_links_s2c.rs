use std::borrow::Cow;
use std::io::Write;

use chunkedge_binary::{Decode, Encode, TextComponent};

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct ServerLinksS2c<'a> {
    pub links: Vec<ServerLink<'a>>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ServerLink<'a> {
    pub label: ServerLinkEnum,
    pub url: Cow<'a, str>,
}

#[derive(Clone, Debug)]
pub enum ServerLinkEnum {
    BuiltIn(BuiltInLinkType),
    CustomText(TextComponent),
}

impl Encode for ServerLinkEnum {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            ServerLinkEnum::BuiltIn(label) => {
                true.encode(&mut w)?;
                label.encode(w)
            }
            ServerLinkEnum::CustomText(label) => {
                false.encode(&mut w)?;
                label.encode(w)
            }
        }
    }
}

impl Decode<'_> for ServerLinkEnum {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        Ok(if bool::decode(r)? {
            ServerLinkEnum::BuiltIn(Decode::decode(r)?)
        } else {
            ServerLinkEnum::CustomText(Decode::decode(r)?)
        })
    }
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
