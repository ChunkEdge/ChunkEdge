use std::borrow::Cow;
use std::io::Write;

use uuid::Uuid;
use valence_binary::{Bounded, Decode, Encode, IdOr, TextComponent, VarInt};
use valence_nbt::Compound;

use crate::{Packet, VariableBitSet};

#[derive(Clone, PartialEq, Debug, Packet)]
pub struct PlayerChatS2c<'a> {
    pub global_index: VarInt,
    pub sender: Uuid,
    pub index: VarInt,
    pub message_signature: Option<&'a [u8; 256]>,
    pub message: Bounded<&'a str, 256>,
    pub timestamp: u64,
    pub salt: u64,
    pub previous_messages: Vec<MessageSignature<'a>>,
    pub unsigned_content: Option<Cow<'a, TextComponent>>,
    pub filter_type: MessageFilterType,
    pub filter_type_bits: Option<VariableBitSet>,
    pub chat_type: ChatType<'a>,
    pub network_name: Cow<'a, TextComponent>,
    pub network_target_name: Option<Cow<'a, TextComponent>>,
}

pub type ChatType<'a> = IdOr<DirectChatType<'a>>;

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct DirectChatType<'a> {
    pub chat: ChatTypeDecoration<'a>,
    pub narration: ChatTypeDecoration<'a>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct ChatTypeDecoration<'a> {
    pub translation_key: Cow<'a, str>,
    pub parameters: Vec<ChatTypeParameter>,
    pub style: Compound,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum ChatTypeParameter {
    Sender,
    Target,
    Content,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum MessageFilterType {
    PassThrough,
    FullyFiltered,
    PartiallyFiltered,
}

impl Encode for PlayerChatS2c<'_> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.global_index.encode(&mut w)?;
        self.sender.encode(&mut w)?;
        self.index.encode(&mut w)?;
        self.message_signature.encode(&mut w)?;
        self.message.encode(&mut w)?;
        self.timestamp.encode(&mut w)?;
        self.salt.encode(&mut w)?;
        self.previous_messages.encode(&mut w)?;
        self.unsigned_content.encode(&mut w)?;
        self.filter_type.encode(&mut w)?;

        if self.filter_type == MessageFilterType::PartiallyFiltered {
            self.filter_type_bits
                .clone()
                .unwrap_or_default()
                .encode(&mut w)?;
        }

        self.chat_type.encode(&mut w)?;
        self.network_name.encode(&mut w)?;
        self.network_target_name.encode(&mut w)?;

        Ok(())
    }
}

impl<'a> Decode<'a> for PlayerChatS2c<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let global_index = VarInt::decode(r)?;
        let sender = Uuid::decode(r)?;
        let index = VarInt::decode(r)?;
        let message_signature = Option::<&'a [u8; 256]>::decode(r)?;
        let message = Decode::decode(r)?;
        let time_stamp = u64::decode(r)?;
        let salt = u64::decode(r)?;
        let previous_messages = Vec::<MessageSignature>::decode(r)?;
        let unsigned_content = Option::<Cow<'a, TextComponent>>::decode(r)?;
        let filter_type = MessageFilterType::decode(r)?;

        let filter_type_bits = match filter_type {
            MessageFilterType::PartiallyFiltered => Some(VariableBitSet::decode(r)?),
            _ => None,
        };

        let chat_type = ChatType::decode(r)?;
        let network_name = <Cow<'a, TextComponent>>::decode(r)?;
        let network_target_name = Option::<Cow<'a, TextComponent>>::decode(r)?;

        Ok(Self {
            global_index,
            sender,
            index,
            message_signature,
            message,
            timestamp: time_stamp,
            salt,
            previous_messages,
            unsigned_content,
            filter_type,
            filter_type_bits,
            chat_type,
            network_name,
            network_target_name,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MessageSignature<'a> {
    pub message_id: i32,
    pub signature: Option<&'a [u8; 256]>,
}

impl Encode for MessageSignature<'_> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        VarInt(self.message_id + 1).encode(&mut w)?;

        match self.signature {
            None => {}
            Some(signature) => signature.encode(&mut w)?,
        }

        Ok(())
    }
}

impl<'a> Decode<'a> for MessageSignature<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let encoded_message_id = VarInt::decode(r)?.0;
        anyhow::ensure!(encoded_message_id != i32::MIN, "message id underflow");
        let message_id = encoded_message_id - 1;

        let signature = if message_id == -1 {
            Some(<&[u8; 256]>::decode(r)?)
        } else {
            None
        };

        Ok(Self {
            message_id,
            signature,
        })
    }
}
