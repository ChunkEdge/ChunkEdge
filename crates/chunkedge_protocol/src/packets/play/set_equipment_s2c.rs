use std::io::Write;

use valence_binary::{Decode, Encode, VarInt};
use valence_item::ItemStack;

use crate::Packet;

#[derive(Clone, PartialEq, Debug, Packet)]
pub struct SetEquipmentS2c {
    pub entity_id: VarInt,
    pub equipment: Vec<EquipmentEntry>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct EquipmentEntry {
    pub slot: EquipmentSlot,
    pub item: ItemStack,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum EquipmentSlot {
    MainHand = 0,
    OffHand = 1,
    Boots = 2,
    Leggings = 3,
    Chestplate = 4,
    Helmet = 5,
    Body = 6,
    Saddle = 7,
}

impl EquipmentSlot {
    pub const fn number_of_members() -> usize {
        // Please update if number changes!!!
        8
    }
}

impl From<u8> for EquipmentSlot {
    fn from(value: u8) -> Self {
        match value {
            0 => EquipmentSlot::MainHand,
            1 => EquipmentSlot::OffHand,
            2 => EquipmentSlot::Boots,
            3 => EquipmentSlot::Leggings,
            4 => EquipmentSlot::Chestplate,
            5 => EquipmentSlot::Helmet,
            6 => EquipmentSlot::Body,
            7 => EquipmentSlot::Saddle,
            _ => panic!("Invalid equipment slot value: {value}"),
        }
    }
}

impl From<i8> for EquipmentSlot {
    fn from(value: i8) -> Self {
        match value {
            0 => EquipmentSlot::MainHand,
            1 => EquipmentSlot::OffHand,
            2 => EquipmentSlot::Boots,
            3 => EquipmentSlot::Leggings,
            4 => EquipmentSlot::Chestplate,
            5 => EquipmentSlot::Helmet,
            6 => EquipmentSlot::Body,
            7 => EquipmentSlot::Saddle,
            _ => panic!("Invalid equipment slot value: {value}"),
        }
    }
}

impl Encode for SetEquipmentS2c {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.entity_id.encode(&mut w)?;

        for i in 0..self.equipment.len() {
            let slot = self.equipment[i].slot as i8;
            if i != self.equipment.len() - 1 {
                (slot | -128).encode(&mut w)?;
            } else {
                slot.encode(&mut w)?;
            }
            self.equipment[i].item.encode(&mut w)?;
        }

        Ok(())
    }
}

impl<'a> Decode<'a> for SetEquipmentS2c {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let entity_id = VarInt::decode(r)?;

        let mut equipment = vec![];

        loop {
            let slot = i8::decode(r)?;
            let item = ItemStack::decode(r)?;
            equipment.push(EquipmentEntry {
                slot: (slot & 127).into(),
                item,
            });
            if slot & -128 == 0 {
                break;
            }
        }

        Ok(Self {
            entity_id,
            equipment,
        })
    }
}
