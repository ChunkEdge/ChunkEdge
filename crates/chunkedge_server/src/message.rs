// TODO: delete this module in favor of chunkedge_chat.

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_protocol::encode::WritePacket;
use chunkedge_protocol::packets::play::{ChatC2s, SystemChatS2c};
use chunkedge_protocol::text::IntoText;
use chunkedge_protocol::IntoTextComponent;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct MessagePlugin;

impl Plugin for MessagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChatReceivedMessage>()
            .add_systems(EventLoopPreUpdate, handle_chat_message);
    }
}

pub trait SendMessage {
    /// Sends a system message visible in the chat.
    fn send_chat_message<'a>(&mut self, msg: impl IntoText<'a>);
    /// Displays a message in the player's action bar (text above the hotbar).
    fn send_action_bar_message<'a>(&mut self, msg: impl IntoText<'a>);
}

impl<T: WritePacket> SendMessage for T {
    fn send_chat_message<'a>(&mut self, msg: impl IntoText<'a>) {
        self.write_packet(&SystemChatS2c {
            chat: msg.into_cow_text_component(),
            overlay: false,
        });
    }

    fn send_action_bar_message<'a>(&mut self, msg: impl IntoText<'a>) {
        self.write_packet(&SystemChatS2c {
            chat: msg.into_cow_text_component(),
            overlay: true,
        });
    }
}

/// Message emitted when a client sends a chat message to the server.
#[derive(Message, Clone, Debug)]
pub struct ChatReceivedMessage {
    /// The client that sent the chat message.
    pub client: Entity,
    /// The raw chat message text sent.
    pub message: Box<str>,
    /// The client-provided timestamp.
    pub timestamp: u64,
}

pub fn handle_chat_message(
    mut packets: MessageReader<PacketMessage>,
    mut messages: MessageWriter<ChatReceivedMessage>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<ChatC2s>() {
            messages.write(ChatReceivedMessage {
                client: packet.client,
                message: pkt.message.0.into(),
                timestamp: pkt.timestamp,
            });
        }
    }
}
