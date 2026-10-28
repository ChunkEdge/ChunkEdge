use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::entity::SharedFlags;
use chunkedge_entity::{Pose, entity};
use chunkedge_protocol::packets::play::PlayerCommandC2s;
pub use chunkedge_protocol::packets::play::player_command_c2s::PlayerCommand;
use chunkedge_protocol::packets::play::player_input_c2s::{PlayerInputC2s, PlayerInputFlags};

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
    mut clients: Query<(&mut entity::Pose, &mut SharedFlags)>,
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

#[derive(Component, Default)]
pub(crate) struct LastPlayerInput(PlayerInputFlags);

/// Tracks sneaking from input, while actual sprinting comes from player commands.
pub(crate) fn handle_player_input(
    mut packets: MessageReader<PacketMessage>,
    mut clients: Query<(&mut entity::Pose, &mut SharedFlags, &mut LastPlayerInput)>,
    mut sneaking_messages: MessageWriter<SneakMessage>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<PlayerInputC2s>() {
            let Ok((mut pose, mut flags, mut last_input)) = clients.get_mut(packet.client) else {
                continue;
            };
            let sneak = pkt.flags.sneak();
            if sneak != last_input.0.sneak() {
                pose.0 = if sneak {
                    Pose::Crouching
                } else {
                    Pose::Standing
                };
                flags.set_sneaking(sneak);
                sneaking_messages.write(SneakMessage {
                    client: packet.client,
                    state: if sneak {
                        SneakState::Start
                    } else {
                        SneakState::Stop
                    },
                });
            }
            last_input.0 = pkt.flags;
        }
    }
}
