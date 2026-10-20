use std::collections::HashMap;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::entity::Flags;
use chunkedge_entity::{Pose, entity};
use chunkedge_protocol::packets::play::PlayerCommandC2s;
pub use chunkedge_protocol::packets::play::player_command_c2s::PlayerCommand;
use chunkedge_protocol::packets::play::player_input_c2s::PlayerInputC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct ClientCommandPlugin;

impl Plugin for ClientCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SprintMessage>()
            .add_message::<SneakMessage>()
            .add_message::<JumpWithHorseMessage>()
            .add_message::<LeaveBedMessage>()
            .add_systems(
                EventLoopPreUpdate,
                (handle_client_command, handle_player_input),
            );
    }
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct SprintMessage {
    pub client: Entity,
    pub state: SprintState,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SprintState {
    Start,
    Stop,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct SneakMessage {
    pub client: Entity,
    pub state: SneakState,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SneakState {
    Start,
    Stop,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct JumpWithHorseMessage {
    pub client: Entity,
    pub state: JumpWithHorseState,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum JumpWithHorseState {
    Start {
        /// The power of the horse jump in `0..=100`.
        power: u8,
    },
    Stop,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct LeaveBedMessage {
    pub client: Entity,
}

fn handle_client_command(
    mut packets: MessageReader<PacketMessage>,
    mut clients: Query<(&mut entity::Pose, &mut Flags)>,
    mut sprinting_messages: MessageWriter<SprintMessage>,
    mut jump_with_horse_messages: MessageWriter<JumpWithHorseMessage>,
    mut leave_bed_messages: MessageWriter<LeaveBedMessage>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<PlayerCommandC2s>() {
            match pkt.action {
                PlayerCommand::StopSleeping => {
                    leave_bed_messages.write(LeaveBedMessage {
                        client: packet.client,
                    });
                }
                PlayerCommand::StartSprinting => {
                    if let Ok((_, mut flags)) = clients.get_mut(packet.client) {
                        flags.set_sprinting(true);
                    }

                    sprinting_messages.write(SprintMessage {
                        client: packet.client,
                        state: SprintState::Start,
                    });
                }
                PlayerCommand::StopSprinting => {
                    if let Ok((_, mut flags)) = clients.get_mut(packet.client) {
                        flags.set_sprinting(false);
                    }

                    sprinting_messages.write(SprintMessage {
                        client: packet.client,
                        state: SprintState::Stop,
                    });
                }
                PlayerCommand::StartRidingJump => {
                    jump_with_horse_messages.write(JumpWithHorseMessage {
                        client: packet.client,
                        state: JumpWithHorseState::Start {
                            power: pkt.jump_boost.0 as u8,
                        },
                    });
                }
                PlayerCommand::StopRidingJump => {
                    jump_with_horse_messages.write(JumpWithHorseMessage {
                        client: packet.client,
                        state: JumpWithHorseState::Stop,
                    });
                }
                PlayerCommand::OpenInventory => {} // TODO
                PlayerCommand::StartFallFlying => {
                    if let Ok((mut pose, _)) = clients.get_mut(packet.client) {
                        pose.0 = Pose::FallFlying;
                    }

                    // TODO.
                }
            }
        }
    }
}

/// Tracks sneak/sprint state from the 26.x player input packet, which carries
/// them as a bitfield instead of the old sneak/sprint command actions.
/// Edge transitions are forwarded as [`SneakMessage`] and [`SprintMessage`].
fn handle_player_input(
    mut packets: MessageReader<PacketMessage>,
    mut clients: Query<(&mut entity::Pose, &mut Flags)>,
    mut sprinting_messages: MessageWriter<SprintMessage>,
    mut sneaking_messages: MessageWriter<SneakMessage>,
    mut last_inputs: Local<HashMap<Entity, (bool, bool)>>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<PlayerInputC2s>() {
            let sneak = pkt.flags.sneak();
            let sprint = pkt.flags.sprint();
            let (was_sneaking, was_sprinting) =
                last_inputs.get(&packet.client).copied().unwrap_or_default();

            if sneak != was_sneaking {
                if let Ok((mut pose, mut flags)) = clients.get_mut(packet.client) {
                    pose.0 = if sneak {
                        Pose::Sneaking
                    } else {
                        Pose::Standing
                    };
                    flags.set_sneaking(sneak);
                }

                sneaking_messages.write(SneakMessage {
                    client: packet.client,
                    state: if sneak {
                        SneakState::Start
                    } else {
                        SneakState::Stop
                    },
                });
            }

            if sprint != was_sprinting {
                if let Ok((_, mut flags)) = clients.get_mut(packet.client) {
                    flags.set_sprinting(sprint);
                }

                sprinting_messages.write(SprintMessage {
                    client: packet.client,
                    state: if sprint {
                        SprintState::Start
                    } else {
                        SprintState::Stop
                    },
                });
            }

            last_inputs.insert(packet.client, (sneak, sprint));
        }
    }
}
