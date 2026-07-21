//! Example demonstrating the packet debugging system with recursive and
//! asymmetrical structures.
//!
//! To run this example and always see the colorized hex dumps and decode trace:
//! ```bash
//! cargo run --example packet_debug_demo --features debug-packets
//! ```
//!
//! To run this example and see traces and hex dumps ONLY when packet decoding
//! fails:
//! ```bash
//! cargo run --example packet_debug_demo --features debug-packets-on-error
//! ```

use std::io::Write;

use chunkedge::protocol::decode::PacketFrame;
use chunkedge::protocol::{Packet, PacketSide, PacketState};
use chunkedge_binary::{Bounded, Decode, Encode};
// To pass ci we convince the unused deps detector we do infact use this dep as the macros do
#[allow(unused_imports)]
use chunkedge_protocol::*;
use chunkedge_protocol_macros::{debug_decode, Packet as DerivePacket};

#[derive(Debug, Encode, Decode, DerivePacket)]
#[packet(id = 100, side = PacketSide::Clientbound)]
enum DeepNestedTree {
    Node {
        name: String,
        left: Box<DeepNestedTree>,
        right: Box<DeepNestedTree>,
    },
    Leaf {
        value: i32,
        tags: Vec<String>,
    },
}

fn run_recursive_scenario() {
    println!("\n=== SCENARIO 1: Recursive & Deeply Nested Structure ===");

    // 1. Construct a valid deeply nested tree
    let valid_tree = DeepNestedTree::Node {
        name: "root".to_owned(),
        left: Box::new(DeepNestedTree::Node {
            name: "child_left".to_owned(),
            left: Box::new(DeepNestedTree::Leaf {
                value: 42,
                tags: vec!["tag1".to_owned(), "tag2".to_owned()],
            }),
            right: Box::new(DeepNestedTree::Leaf {
                value: 99,
                tags: vec![],
            }),
        }),
        right: Box::new(DeepNestedTree::Leaf {
            value: 7,
            tags: vec!["lone_tag".to_owned()],
        }),
    };

    // Serialize it
    let mut valid_bytes = Vec::new();
    valid_tree.encode(&mut valid_bytes).unwrap();

    // Decode it (Success)
    println!("\n1a. Decoding a valid deeply nested tree (Should succeed and trace fields):");
    let success_frame = PacketFrame {
        id: DeepNestedTree::ID,
        body: valid_bytes.as_slice().into(),
    };
    let _decoded_success: DeepNestedTree = success_frame.decode().unwrap();

    // 2. Corrupt a nested field to show deep trace failure
    println!(
        "\n1b. Decoding a corrupted nested tree (Should fail and show the exact error location):"
    );
    let mut corrupted_bytes = valid_bytes;

    // Find the sequence representing "tag1" (prefixed by its length 4)
    // and replace length 4 with 100 to cause a bounds check error during decode.
    if let Some(pos) = corrupted_bytes
        .windows(5)
        .position(|w| w == [4, b't', b'a', b'g', b'1'])
    {
        corrupted_bytes[pos] = 100;
    }

    let failure_frame = PacketFrame {
        id: DeepNestedTree::ID,
        body: corrupted_bytes.as_slice().into(),
    };
    let _decoded_failure: Result<DeepNestedTree, _> = failure_frame.decode();
}

#[derive(Debug)]
struct AsymmetricPayload {
    magic: u32,
    data: String,
    checksum: u8,
}

impl Encode for AsymmetricPayload {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.magic.encode(&mut w)?;
        self.data.encode(&mut w)?;
        self.checksum.encode(w)
    }
}

#[debug_decode]
impl<'a> Decode<'a> for AsymmetricPayload {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let magic = u32::decode(r)?;
        anyhow::ensure!(magic == 0xcafebabe, "invalid magic header: {magic:#x}");

        let data = String::decode(r)?;
        let checksum = u8::decode(r)?;

        // Custom checksum calculation (sum of data bytes modulo 256)
        let calculated = data
            .as_bytes()
            .iter()
            .fold(0_u8, |acc, &b| acc.wrapping_add(b));
        anyhow::ensure!(
            checksum == calculated,
            "checksum mismatch: expected {calculated:#x}, got {checksum:#x}"
        );

        Ok(Self {
            magic,
            data,
            checksum,
        })
    }
}

impl Packet for AsymmetricPayload {
    const ID: i32 = 101;
    const NAME: &'static str = "AsymmetricPayload";
    const SIDE: PacketSide = PacketSide::Clientbound;
    const STATE: PacketState = PacketState::Play;
}

fn run_asymmetric_scenario() {
    println!("\n=== SCENARIO 2: Asymmetric Struct with Manual Decode Validation ===");

    // Construct a valid payload
    let data = "Hello Debugger!".to_owned();
    let checksum = data
        .as_bytes()
        .iter()
        .fold(0_u8, |acc, &b| acc.wrapping_add(b));
    let valid_payload = AsymmetricPayload {
        magic: 0xcafebabe,
        data,
        checksum,
    };

    let mut valid_bytes = Vec::new();
    valid_payload.encode(&mut valid_bytes).unwrap();

    // Decode Valid
    println!("\n2a. Decoding valid AsymmetricPayload:");
    let success_frame = PacketFrame {
        id: AsymmetricPayload::ID,
        body: valid_bytes.as_slice().into(),
    };
    let _decoded_success: AsymmetricPayload = success_frame.decode().unwrap();

    // Decode with wrong magic header
    println!("\n2b. Decoding AsymmetricPayload with wrong magic header:");
    let mut wrong_magic_bytes = valid_bytes.clone();
    wrong_magic_bytes[0..4].copy_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
    let wrong_magic_frame = PacketFrame {
        id: AsymmetricPayload::ID,
        body: wrong_magic_bytes.as_slice().into(),
    };
    let _decoded_magic_failure: Result<AsymmetricPayload, _> = wrong_magic_frame.decode();

    // Decode with wrong checksum
    println!("\n2c. Decoding AsymmetricPayload with corrupted checksum:");
    let mut wrong_checksum_bytes = valid_bytes;
    let len = wrong_checksum_bytes.len();
    wrong_checksum_bytes[len - 1] ^= 0xff; // Corrupt the checksum byte
    let wrong_checksum_frame = PacketFrame {
        id: AsymmetricPayload::ID,
        body: wrong_checksum_bytes.as_slice().into(),
    };
    let _decoded_checksum_failure: Result<AsymmetricPayload, _> = wrong_checksum_frame.decode();
}

#[derive(Debug, Encode, Decode, DerivePacket)]
#[packet(id = 102, side = PacketSide::Clientbound)]
struct BoundedListPacket {
    items: Bounded<Vec<i32>, 3>,
}

// Helper struct with the same layout but without the Bounded constraint for
// serializing too many items.
#[derive(Debug, Encode, DerivePacket)]
#[packet(id = 102, side = PacketSide::Clientbound)]
struct UnboundedListPacket {
    items: Vec<i32>,
}

fn run_bounded_scenario() {
    println!("\n=== SCENARIO 3: Built-in Bounded Length Verification ===");

    // 1. Valid case (3 items)
    let valid_packet = UnboundedListPacket {
        items: vec![10, 20, 30],
    };
    let mut valid_bytes = Vec::new();
    valid_packet.encode(&mut valid_bytes).unwrap();

    println!("\n3a. Decoding BoundedListPacket with 3 items (Should succeed):");
    let success_frame = PacketFrame {
        id: BoundedListPacket::ID,
        body: valid_bytes.as_slice().into(),
    };
    let _decoded_success: BoundedListPacket = success_frame.decode().unwrap();

    // 2. Invalid case (5 items)
    let invalid_packet = UnboundedListPacket {
        items: vec![10, 20, 30, 40, 50],
    };
    let mut invalid_bytes = Vec::new();
    invalid_packet.encode(&mut invalid_bytes).unwrap();

    println!("\n3b. Decoding BoundedListPacket with 5 items (Should fail Bounded limit of 3):");
    let failure_frame = PacketFrame {
        id: BoundedListPacket::ID,
        body: invalid_bytes.as_slice().into(),
    };
    let _decoded_failure: Result<BoundedListPacket, _> = failure_frame.decode();
}

// =============================================================================
// Main Entry Point
// =============================================================================

pub fn main() {
    run_recursive_scenario();
    run_asymmetric_scenario();
    run_bounded_scenario();
}
