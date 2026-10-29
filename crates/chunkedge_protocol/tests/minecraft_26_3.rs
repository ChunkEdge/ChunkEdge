//! Wire fixtures transcribed from the Minecraft 26.3 stream codecs.
#[cfg(test)]
mod tests {
    use chunkedge_binary::{Decode, Encode, VarInt};
    use chunkedge_math::{DVec3, Vec3};
    use chunkedge_protocol::entity_movement::{DeltaStep, PositionDelta, PositionPath};
    use chunkedge_protocol::game_mode::OptGameMode;
    use chunkedge_protocol::packets::play::level_particles_s2c::ParticleRandomization;
    use chunkedge_protocol::packets::play::{
        EntityPositionSyncS2c, LevelParticlesS2c, MoveEntityPosRotS2c, MoveEntityPosS2c,
        MoveEntityRotS2c, PunchC2s,
    };
    use chunkedge_protocol::{ByteAngle, GameMode, Particle};

    fn bytes(value: &impl Encode) -> Vec<u8> {
        let mut out = Vec::new();
        value.encode(&mut out).unwrap();
        out
    }

    #[test]
    fn relative_movement_properties_precede_deltas_and_angles() {
        let fixture = [7, 1, 0, 1, 0xff, 0xfe, 0, 3, 64, 128];
        let packet = MoveEntityPosRotS2c {
            entity_id: VarInt(7),
            delta: PositionDelta::Linear([1, -2, 3]),
            yaw: ByteAngle(64),
            pitch: ByteAngle(128),
            on_ground: true,
        };
        assert_eq!(bytes(&packet), fixture);
        let mut input = fixture.as_slice();
        let decoded = MoveEntityPosRotS2c::decode(&mut input).unwrap();
        assert_eq!(decoded.delta, packet.delta);
        assert!(decoded.on_ground && input.is_empty());
        assert_eq!(
            bytes(&MoveEntityRotS2c {
                entity_id: VarInt(7),
                on_ground: true,
                yaw: ByteAngle(64),
                pitch: ByteAngle(128)
            }),
            [7, 1, 64, 128]
        );
    }

    #[test]
    fn timed_relative_steps_have_no_separate_list_length() {
        let fixture = [7, 3, 5, 0, 1, 0, 2, 0, 3];
        let packet = MoveEntityPosS2c {
            entity_id: VarInt(7),
            on_ground: true,
            delta: PositionDelta::Stepped(vec![DeltaStep {
                tick_offset: VarInt(5),
                delta: [1, 2, 3],
            }]),
        };
        assert_eq!(bytes(&packet), fixture);
        for len in 0..fixture.len() {
            assert!(MoveEntityPosS2c::decode(&mut &fixture[..len]).is_err());
        }
        assert!(MoveEntityPosS2c::decode(&mut &[7, 0xfe, 0xff, 0xff, 0xff, 0x0f][..]).is_err());
    }

    #[test]
    fn absolute_sync_contains_a_path_and_no_velocity() {
        let packet = EntityPositionSyncS2c {
            entity_id: VarInt(7),
            position: PositionPath::Linear(DVec3::ZERO),
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
        };
        let mut expected = vec![7, 0];
        expected.extend([0; 32]); // Three position doubles and two angle floats.
        expected.push(1);
        assert_eq!(bytes(&packet), expected);
    }

    #[test]
    fn previous_game_mode_is_an_optional_varint() {
        assert_eq!(bytes(&OptGameMode(None)), [0]);
        assert_eq!(bytes(&OptGameMode(Some(GameMode::Survival))), [1]);
        assert_eq!(bytes(&OptGameMode(Some(GameMode::Spectator))), [4]);
        assert_eq!(
            OptGameMode::decode(&mut &[4][..]).unwrap().0,
            Some(GameMode::Spectator)
        );
        assert!(bytes(&PunchC2s).is_empty());
    }

    #[test]
    fn particle_type_is_first_and_count_is_a_varint() {
        let packet = LevelParticlesS2c {
            particle: Particle::AngryVillager,
            long_distance: true,
            always_visible: false,
            position: DVec3::ZERO,
            offset: Vec3::ZERO,
            max_speed: Vec3::ZERO,
            count: VarInt(300),
            randomization: ParticleRandomization::AlternativeWithSpeed,
        };
        let mut expected = bytes(&packet.particle);
        expected.extend([1, 0]);
        expected.extend([0; 48]); // Three doubles and six floats.
        expected.extend([0xac, 2, 2]);
        assert_eq!(bytes(&packet), expected);
        let mut input = expected.as_slice();
        let decoded = LevelParticlesS2c::decode(&mut input).unwrap();
        assert_eq!(decoded.count.0, 300);
        assert!(input.is_empty());
    }

    #[test]
    fn variable_bitsets_use_little_endian_bytes() {
        let mut bits = chunkedge_protocol::VariableBitSet::new();
        bits.set(0);
        bits.set(70);
        let expected = [9, 1, 0, 0, 0, 0, 0, 0, 0, 64];
        assert_eq!(bytes(&bits), expected);
        let mut input = expected.as_slice();
        assert_eq!(
            chunkedge_protocol::VariableBitSet::decode(&mut input).unwrap(),
            bits
        );
        assert!(input.is_empty());
    }

    #[test]
    fn registry_data_accepts_list_entries() {
        use chunkedge_nbt::{List, Value};
        use chunkedge_protocol::packets::configuration::RegistryDataS2c;
        let packet = RegistryDataS2c {
            id: chunkedge_protocol::ident!("block_transformer").into(),
            entries: [(
                chunkedge_protocol::ident!("axe").into(),
                Some(Value::List(List::Compound(vec![]))),
            )]
            .into_iter()
            .collect(),
        };
        let encoded = bytes(&packet);
        // Present flag, TAG_List, compound element type, and a zero element count.
        assert!(encoded.ends_with(&[1, 9, 10, 0, 0, 0, 0]));
        let mut input = encoded.as_slice();
        let decoded = RegistryDataS2c::decode(&mut input).unwrap();
        assert_eq!(decoded.entries, packet.entries);
        assert!(input.is_empty());
    }
}
