#![doc = include_str!("../README.md")]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
mod interaction_broadcast;
pub use interaction_broadcast::EquipmentInteractionBroadcast;
mod inventory_sync;
pub use inventory_sync::EquipmentInventorySync;
use valence_server::client::{Client, FlushPacketsSet, LoadEntityForClientEvent};
use valence_server::entity::living::LivingEntity;
use valence_server::entity::{EntityId, EntityLayerId, Position};
use valence_server::protocol::packets::play::set_equipment_s2c::{EquipmentEntry, EquipmentSlot};
use valence_server::protocol::packets::play::SetEquipmentS2c;
use valence_server::protocol::WritePacket;
use valence_server::{EntityLayer, ItemStack, Layer};

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                on_entity_init,
                interaction_broadcast::start_interaction,
                interaction_broadcast::stop_interaction,
                inventory_sync::on_attach_inventory_sync,
                inventory_sync::equipment_inventory_sync,
                inventory_sync::equipment_held_item_sync_from_client,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                update_equipment.before(FlushPacketsSet),
                on_entity_load.before(FlushPacketsSet),
            ),
        )
        .add_event::<EquipmentChangeEvent>();
    }
}

/// Contains the visible equipment of a [`LivingEntity`], such as armor and held
/// items. By default this is not synced with a player's
/// [`valence_inventory::Inventory`], so the armor the player has equipped in
/// their inventory, will not be visible by other players. You would have to
/// change the equipment in this component here or attach the
/// [`EquipmentInventorySync`] component to the player entity to sync the
/// equipment with the inventory.
#[derive(Debug, Default, Clone, Component)]
pub struct Equipment {
    equipment: [ItemStack; Self::SLOT_COUNT],
    /// Contains a set bit for each modified slot in `slots`.
    #[doc(hidden)]
    pub(crate) changed: u8,
}

#[allow(clippy::large_stack_arrays)] // About 10kb but I think its worth it.
impl Equipment {
    pub const SLOT_COUNT: usize = EquipmentSlot::number_of_members();

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        main_hand: ItemStack,
        off_hand: ItemStack,
        boots: ItemStack,
        leggings: ItemStack,
        chestplate: ItemStack,
        helmet: ItemStack,
        body: ItemStack,
        saddle: ItemStack,
    ) -> Self {
        let equipment = [
            main_hand, off_hand, boots, leggings, chestplate, helmet, body, saddle,
        ];
        Self {
            equipment,
            changed: 0,
        }
    }

    pub fn new_empty() -> Self {
        let equipment = [ItemStack::EMPTY; 8];
        Self {
            equipment,
            changed: 0,
        }
    }

    pub fn slot(&self, idx: EquipmentSlot) -> &ItemStack {
        &self.equipment[idx as usize]
    }

    pub fn set_slot(&mut self, idx: EquipmentSlot, item: ItemStack) {
        if self.equipment[idx as usize] != item {
            self.equipment[idx as usize] = item;
            self.changed |= 1 << idx as u8;
        }
    }

    pub fn main_hand(&self) -> &ItemStack {
        self.slot(EquipmentSlot::MainHand)
    }

    pub fn off_hand(&self) -> &ItemStack {
        self.slot(EquipmentSlot::OffHand)
    }

    pub fn feet(&self) -> &ItemStack {
        self.slot(EquipmentSlot::Boots)
    }

    pub fn legs(&self) -> &ItemStack {
        self.slot(EquipmentSlot::Leggings)
    }

    pub fn chest(&self) -> &ItemStack {
        self.slot(EquipmentSlot::Chestplate)
    }

    pub fn head(&self) -> &ItemStack {
        self.slot(EquipmentSlot::Helmet)
    }

    pub fn body(&self) -> &ItemStack {
        self.slot(EquipmentSlot::Body)
    }

    pub fn saddle(&self) -> &ItemStack {
        self.slot(EquipmentSlot::Saddle)
    }

    pub fn set_main_hand(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::MainHand, item);
    }

    pub fn set_off_hand(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::OffHand, item);
    }

    pub fn set_feet(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::Boots, item);
    }

    pub fn set_legs(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::Leggings, item);
    }

    pub fn set_chest(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::Chestplate, item);
    }

    pub fn set_head(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::Helmet, item);
    }

    pub fn set_body(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::Body, item);
    }

    pub fn set_saddle(&mut self, item: ItemStack) {
        self.set_slot(EquipmentSlot::Saddle, item);
    }

    pub fn clear(&mut self) {
        self.equipment
            .iter_mut()
            .for_each(|itm| *itm = ItemStack::EMPTY);
        self.changed = u8::MAX
    }

    pub fn is_default(&self) -> bool {
        self.equipment.iter().all(|item| item.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct EquipmentSlotChange {
    idx: u8,
    stack: ItemStack,
}

#[derive(Debug, Clone, Event)]
pub struct EquipmentChangeEvent {
    pub client: Entity,
    pub changed: Vec<EquipmentSlotChange>,
}

fn update_equipment(
    mut clients: Query<
        (Entity, &EntityId, &EntityLayerId, &Position, &mut Equipment),
        Changed<Equipment>,
    >,
    mut event_writer: EventWriter<EquipmentChangeEvent>,
    mut entity_layer: Query<&mut EntityLayer>,
) {
    for (entity, entity_id, entity_layer_id, position, mut equipment) in &mut clients {
        let Ok(mut entity_layer) = entity_layer.get_mut(entity_layer_id.0) else {
            continue;
        };

        if equipment.changed != 0 {
            let mut slots_changed: Vec<EquipmentSlotChange> =
                Vec::with_capacity(Equipment::SLOT_COUNT);

            for slot in 0..Equipment::SLOT_COUNT {
                if equipment.changed & (1 << slot) != 0 {
                    slots_changed.push(EquipmentSlotChange {
                        idx: slot as u8,
                        stack: equipment.equipment[slot].clone(),
                    });
                }
            }

            entity_layer
                .view_except_writer(position.0, entity)
                .write_packet(&SetEquipmentS2c {
                    entity_id: entity_id.get().into(),
                    equipment: slots_changed
                        .iter()
                        .map(|change| EquipmentEntry {
                            slot: change.idx.into(),
                            item: change.stack.clone(),
                        })
                        .collect(),
                });

            event_writer.send(EquipmentChangeEvent {
                client: entity,
                changed: slots_changed,
            });

            equipment.changed = 0;
        }
    }
}

/// Gets called when the player loads an entity, for example
/// when the player gets in range of the entity.
fn on_entity_load(
    mut clients: Query<&mut Client>,
    entities: Query<(&EntityId, &Equipment)>,
    mut events: EventReader<LoadEntityForClientEvent>,
) {
    for event in events.read() {
        let Ok(mut client) = clients.get_mut(event.client) else {
            continue;
        };

        let Ok((entity_id, equipment)) = entities.get(event.entity_loaded) else {
            continue;
        };

        if equipment.is_default() {
            continue;
        }

        let mut entries: Vec<EquipmentEntry> = Vec::with_capacity(Equipment::SLOT_COUNT);
        for (idx, stack) in equipment.equipment.iter().enumerate() {
            entries.push(EquipmentEntry {
                slot: (idx as u8).into(),
                item: stack.clone(),
            });
        }

        client.write_packet(&SetEquipmentS2c {
            entity_id: entity_id.get().into(),
            equipment: entries,
        });
    }
}

/// Add a default equipment component to all living entities when they are
/// initialized.
fn on_entity_init(
    mut commands: Commands,
    mut entities: Query<Entity, (Added<LivingEntity>, Without<Equipment>)>,
) {
    for entity in &mut entities {
        commands.entity(entity).insert(Equipment::default());
    }
}
