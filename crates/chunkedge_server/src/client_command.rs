use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::entity::Flags;
use chunkedge_entity::{entity, Pose};
pub use chunkedge_protocol::packets::play::player_command_c2s::PlayerCommand;
use chunkedge_protocol::packets::play::PlayerCommandC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketEvent};

pub struct ClientCommandPlugin;

impl Plugin for ClientCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SprintEvent>()
            .add_message::<SneakEvent>()
            .add_message::<JumpWithHorseEvent>()
            .add_message::<LeaveBedEvent>()
            .add_systems(EventLoopPreUpdate, handle_client_command);
    }
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct SprintEvent {
    pub client: Entity,
    pub state: SprintState,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SprintState {
    Start,
    Stop,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct SneakEvent {
    pub client: Entity,
    pub state: SneakState,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SneakState {
    Start,
    Stop,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct JumpWithHorseEvent {
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
pub struct LeaveBedEvent {
    pub client: Entity,
}

fn handle_client_command(
    mut packets: MessageReader<PacketEvent>,
    mut clients: Query<(&mut entity::Pose, &mut Flags)>,
    mut sprinting_events: MessageWriter<SprintEvent>,
    mut sneaking_events: MessageWriter<SneakEvent>,
    mut jump_with_horse_events: MessageWriter<JumpWithHorseEvent>,
    mut leave_bed_events: MessageWriter<LeaveBedEvent>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<PlayerCommandC2s>() {
            match pkt.action {
                PlayerCommand::StartSneaking => {
                    if let Ok((mut pose, mut flags)) = clients.get_mut(packet.client) {
                        pose.0 = Pose::Sneaking;
                        flags.set_sneaking(true);
                    }

                    sneaking_events.write(SneakEvent {
                        client: packet.client,
                        state: SneakState::Start,
                    });
                }
                PlayerCommand::StopSneaking => {
                    if let Ok((mut pose, mut flags)) = clients.get_mut(packet.client) {
                        pose.0 = Pose::Standing;
                        flags.set_sneaking(false);
                    }

                    sneaking_events.write(SneakEvent {
                        client: packet.client,
                        state: SneakState::Stop,
                    });
                }
                PlayerCommand::LeaveBed => {
                    leave_bed_events.write(LeaveBedEvent {
                        client: packet.client,
                    });
                }
                PlayerCommand::StartSprinting => {
                    if let Ok((_, mut flags)) = clients.get_mut(packet.client) {
                        flags.set_sprinting(true);
                    }

                    sprinting_events.write(SprintEvent {
                        client: packet.client,
                        state: SprintState::Start,
                    });
                }
                PlayerCommand::StopSprinting => {
                    if let Ok((_, mut flags)) = clients.get_mut(packet.client) {
                        flags.set_sprinting(false);
                    }

                    sprinting_events.write(SprintEvent {
                        client: packet.client,
                        state: SprintState::Stop,
                    });
                }
                PlayerCommand::StartJumpWithHorse => {
                    jump_with_horse_events.write(JumpWithHorseEvent {
                        client: packet.client,
                        state: JumpWithHorseState::Start {
                            power: pkt.jump_boost.0 as u8,
                        },
                    });
                }
                PlayerCommand::StopJumpWithHorse => {
                    jump_with_horse_events.write(JumpWithHorseEvent {
                        client: packet.client,
                        state: JumpWithHorseState::Stop,
                    });
                }
                PlayerCommand::OpenHorseInventory => {} // TODO
                PlayerCommand::StartFlyingWithElytra => {
                    if let Ok((mut pose, _)) = clients.get_mut(packet.client) {
                        pose.0 = Pose::FallFlying;
                    }

                    // TODO.
                }
            }
        }
    }
}
