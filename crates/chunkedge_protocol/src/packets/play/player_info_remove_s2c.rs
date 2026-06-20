use std::borrow::Cow;

use uuid::Uuid;
use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, PartialEq, Debug, Encode, Decode, Packet)]
pub struct PlayerInfoRemoveS2c<'a> {
    pub uuids: Cow<'a, [Uuid]>,
}
