#![doc = include_str!("../README.md")]
#![allow(deprecated)] // TODO: update aes library

/// Used only by macros. Not public API.
#[doc(hidden)]
pub mod __private {
    pub use anyhow::{anyhow, bail, ensure, Context, Result};

    pub use crate::Packet;
}

extern crate self as valence_protocol;

mod biome_pos;
mod bit_storage;
pub mod chunk_pos;
pub mod chunk_section_pos;
pub mod decode;
mod difficulty;
pub mod encode;
pub mod game_mode;
mod global_pos;
mod hand;
pub mod movement_flags;
pub mod packets;
pub mod profile;
pub mod sound;
mod velocity;

use std::io::Write;

use anyhow::Context;
pub use biome_pos::BiomePos;
pub use bit_storage::BitStorage;
pub use block::{BlockKind, BlockState};
pub use valence_binary::block_pos::BlockPos;
pub use chunk_pos::ChunkPos;
pub use chunk_section_pos::ChunkSectionPos;
pub use decode::PacketDecoder;
use derive_more::{From, Into};
pub use difficulty::Difficulty;
pub use valence_binary::direction::Direction;
pub use encode::{PacketEncoder, WritePacket};
pub use game_mode::GameMode;
pub use global_pos::GlobalPos;
pub use hand::Hand;
pub use ident::ident;
pub use packets::play::level_particles_s2c::Particle;
use serde::{Deserialize, Serialize};
pub use sound::Sound;
pub use text::{JsonText, Text};
pub use valence_binary::array::FixedArray;
pub use valence_binary::bit_set::FixedBitSet;
pub use valence_binary::byte_angle::ByteAngle;
use valence_binary::Encode;
pub use valence_binary::{
    IDSet, IdOr, IntoTextComponent, TextComponent, VarInt, VarIntDecodeError, VarLong,
};
pub use valence_generated::registry_id::RegistryId;
pub use valence_generated::{block, packet_id, status_effects};
pub use valence_ident::Ident;
pub use valence_item::{ItemKind, ItemStack};
use valence_protocol_macros::Packet;
pub use velocity::Velocity;
pub use {
    anyhow, bytes, uuid, valence_ident as ident, valence_math as math, valence_nbt as nbt,
    valence_text as text,
};

/// The maximum number of bytes in a single Minecraft packet.
pub const MAX_PACKET_SIZE: i32 = 2097152;

/// The Minecraft protocol version this library currently targets.
pub const PROTOCOL_VERSION: i32 = 770;

/// The stringified name of the Minecraft version this library currently
/// targets.
pub const MINECRAFT_VERSION: &str = "1.21.5";

/// How large a packet should be before it is compressed by the packet encoder.
///
/// If the inner value is >= 0, then packets with encoded lengths >= to this
/// value will be compressed. If the value is negative, then compression is
/// disabled and no packets are compressed.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct CompressionThreshold(pub i32);

impl CompressionThreshold {
    /// No compression.
    pub const DEFAULT: Self = Self(-1);
}

/// No compression.
impl Default for CompressionThreshold {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Types considered to be Minecraft packets.
///
/// In serialized form, a packet begins with a [`VarInt`] packet ID followed by
/// the body of the packet. If present, the implementations of [`Encode`] and
/// [`valence_binary::Decode`] on `Self` are expected to only encode/decode the _body_ of this
/// packet without the leading ID.
pub trait Packet: std::fmt::Debug {
    /// The leading `VarInt` ID of this packet.
    const ID: i32;
    /// The name of this packet for debugging purposes.
    const NAME: &'static str;
    /// The side this packet is intended for.
    const SIDE: PacketSide;
    /// The state in which this packet is used.
    const STATE: PacketState;

    /// Encodes this packet's `VarInt` ID first, followed by the packet's body.
    fn encode_with_id(&self, mut w: impl Write) -> anyhow::Result<()>
    where
        Self: Encode,
    {
        VarInt(Self::ID)
            .encode(&mut w)
            .context("failed to encode packet ID")?;

        self.encode(w)
    }
}

/// The side a packet is intended for.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PacketSide {
    /// Server -> Client
    Clientbound,
    /// Client -> Server
    Serverbound,
}

/// The state in  which a packet is used.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PacketState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use bytes::BytesMut;
    // use crate::{Packet, PacketSide};
    use valence_binary::{Decode, Encode, VarInt, VarLong};
    use valence_item::{ItemKind, ItemStack};
    use valence_protocol_macros::Packet;

    use super::*;
    use valence_binary::block_pos::BlockPos;
    use crate::decode::PacketDecoder;
    use crate::encode::PacketEncoder;
    use crate::hand::Hand;
    use crate::text::{IntoText, Text};
    use crate::Ident;

    #[derive(Encode, Decode, Packet, Debug)]
    #[packet(id = 1, side = PacketSide::Clientbound)]
    struct RegularStruct {
        foo: i32,
        bar: bool,
        baz: f64,
    }

    #[derive(Encode, Decode, Packet, Debug)]
    #[packet(id = 2, side = PacketSide::Clientbound)]
    struct UnitStruct;

    #[derive(Encode, Decode, Packet, Debug)]
    #[packet(id = 3, side = PacketSide::Clientbound)]
    struct EmptyStruct;

    #[derive(Encode, Decode, Packet, Debug)]
    #[packet(id = 4, side = PacketSide::Clientbound)]
    struct TupleStruct(i32, bool, f64);

    #[derive(Encode, Decode, Packet, Debug)]
    #[packet(id = 5, side = PacketSide::Clientbound)]
    struct StructWithGenerics<'z, T = ()> {
        foo: &'z str,
        bar: T,
    }

    #[derive(Encode, Decode, Packet, Debug)]
    #[packet(id = 6, side = PacketSide::Clientbound)]
    struct TupleStructWithGenerics<'z, T = ()>(&'z str, i32, T);

    #[allow(unconditional_recursion, clippy::extra_unused_type_parameters)]
    fn assert_has_impls<'a, T>()
    where
        T: Encode + Decode<'a> + Packet,
    {
        assert_has_impls::<RegularStruct>();
        assert_has_impls::<UnitStruct>();
        assert_has_impls::<EmptyStruct>();
        assert_has_impls::<TupleStruct>();
        assert_has_impls::<StructWithGenerics>();
        assert_has_impls::<TupleStructWithGenerics>();
    }

    #[test]
    fn packet_name() {
        assert_eq!(RegularStruct::NAME, "RegularStruct");
        assert_eq!(UnitStruct::NAME, "UnitStruct");
        assert_eq!(StructWithGenerics::<()>::NAME, "StructWithGenerics");
    }

    #[cfg(feature = "encryption")]
    const CRYPT_KEY: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    #[derive(PartialEq, Debug, Encode, Decode, Packet)]
    #[packet(id = 42, side = PacketSide::Clientbound)]
    struct TestPacket<'a> {
        a: bool,
        b: u8,
        c: i32,
        d: f32,
        e: f64,
        f: BlockPos,
        g: Hand,
        h: Ident<Cow<'a, str>>,
        i: ItemStack,
        j: Text,
        k: VarInt,
        l: VarLong,
        m: &'a str,
        n: &'a [u8; 10],
        o: [u128; 3],
    }

    impl<'a> TestPacket<'a> {
        fn new(string: &'a str) -> Self {
            Self {
                a: true,
                b: 12,
                c: -999,
                d: 5.001,
                e: 1e10,
                f: BlockPos::new(1, 2, 3),
                g: Hand::Off,
                h: Ident::new("minecraft:whatever").unwrap(),
                i: ItemStack::new(ItemKind::WoodenSword, 12),
                j: "my ".into_text() + "fancy".italic() + " text",
                k: VarInt(123),
                l: VarLong(456),
                m: string,
                n: &[7; 10],
                o: [123456789; 3],
            }
        }
    }

    fn check_test_packet(dec: &mut PacketDecoder, string: &str) {
        let frame = dec.try_next_packet().unwrap().unwrap();

        let pkt = frame.decode::<TestPacket>().unwrap();

        assert_eq!(&pkt, &TestPacket::new(string));
    }

    #[test]
    fn packets_round_trip() {
        let mut buf = BytesMut::new();

        let mut enc = PacketEncoder::new();

        enc.append_packet(&TestPacket::new("first")).unwrap();
        #[cfg(feature = "compression")]
        enc.set_compression(0.into());
        enc.append_packet(&TestPacket::new("second")).unwrap();
        buf.unsplit(enc.take());
        #[cfg(feature = "encryption")]
        enc.enable_encryption(&CRYPT_KEY);
        enc.append_packet(&TestPacket::new("third")).unwrap();
        enc.prepend_packet(&TestPacket::new("fourth")).unwrap();

        buf.unsplit(enc.take());

        let mut dec = PacketDecoder::new();

        dec.queue_bytes(buf);

        check_test_packet(&mut dec, "first");

        #[cfg(feature = "compression")]
        dec.set_compression(0.into());

        check_test_packet(&mut dec, "second");

        #[cfg(feature = "encryption")]
        dec.enable_encryption(&CRYPT_KEY);

        check_test_packet(&mut dec, "fourth");
        check_test_packet(&mut dec, "third");
    }
}
