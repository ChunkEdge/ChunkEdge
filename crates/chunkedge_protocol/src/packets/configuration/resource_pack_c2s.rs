use uuid::Uuid;
use valence_binary::{Decode, Encode};

use crate::packets::play::resource_pack_c2s::ResourcePackStatus;
use crate::{Packet, PacketState};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct ResourcePackC2s {
    uuid: Uuid,
    result: ResourcePackStatus,
}
