use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use uuid::Uuid;

use crate::Packet;

#[derive(Clone, PartialEq, Debug, Encode, Decode, Packet)]
pub struct PlayerInfoRemoveS2c<'a> {
    pub uuids: Cow<'a, [Uuid]>,
}
