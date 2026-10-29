use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use chunkedge_entity::EntityAnimations;
use chunkedge_item::{SwingAnimation, SwingAnimationType};
use chunkedge_protocol::Hand;
use chunkedge_protocol::packets::play::PunchC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct HandSwingPlugin;

impl Plugin for HandSwingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HandSwingMessage>()
            .add_systems(EventLoopPreUpdate, handle_hand_swing);
    }
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct HandSwingMessage {
    pub client: Entity,
    pub hand: Hand,
}

/// Queues a hand animation and reports it through [`HandSwingMessage`].
#[derive(SystemParam)]
pub struct HandSwingWriter<'w, 's> {
    clients: Query<'w, 's, &'static mut EntityAnimations>,
    messages: MessageWriter<'w, HandSwingMessage>,
}

impl HandSwingWriter<'_, '_> {
    /// Swings with the default animation, returning false if the entity is missing.
    pub fn swing(&mut self, client: Entity, hand: Hand) -> bool {
        // TODO: the kind and duration of this should not be hardcoded.
        // We probably need to get this from the item in the player's hand
        // but this should be done after item component defaults are implemented
        // for now, hardcoding it as below is fine.
        self.swing_with_animation(
            client,
            hand,
            SwingAnimation {
                kind: SwingAnimationType::Whack,
                duration: chunkedge_protocol::VarInt(6),
            },
        )
    }

    /// Swings with an explicit animation (for example, an item's interact animation).
    pub fn swing_with_animation(
        &mut self,
        client: Entity,
        hand: Hand,
        animation: SwingAnimation,
    ) -> bool {
        let Ok(mut animations) = self.clients.get_mut(client) else {
            return false;
        };
        animations.trigger_swing(hand, animation);
        self.messages.write(HandSwingMessage { client, hand });
        true
    }
}

fn handle_hand_swing(mut packets: MessageReader<PacketMessage>, mut swings: HandSwingWriter) {
    for packet in packets.read() {
        if packet.decode::<PunchC2s>().is_some() {
            swings.swing(packet.client, Hand::Main);
        }
    }
}
