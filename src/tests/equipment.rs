use chunkedge_equipment::{Equipment, EquipmentInventorySync};
use chunkedge_inventory::player_inventory::PlayerInventory;
use chunkedge_inventory::{ClickMode, ClientInventoryState, Inventory};
use chunkedge_item::HashedItemStack;
use chunkedge_server::entity::armor_stand::ArmorStandEntityBundle;
use chunkedge_server::entity::item::ItemEntityBundle;
use chunkedge_server::entity::zombie::ZombieEntityBundle;
use chunkedge_server::entity::{EntityLayerId, Position};
use chunkedge_server::math::DVec3;
use chunkedge_server::protocol::packets::play::container_click_c2s::HashedSlotChange;
use chunkedge_server::protocol::packets::play::{
    ContainerClickC2s, SetCarriedItemC2s, SetEquipmentS2c,
};
use chunkedge_server::{ItemKind, ItemStack};

use crate::testing::ScenarioSingleClient;

#[test]
fn test_only_send_update_to_other_players() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let mut player_equipment = app
        .world_mut()
        .get_mut::<Equipment>(client)
        .expect("could not get player equipment");

    player_equipment.set_chest(ItemStack::new(ItemKind::DiamondChestplate, 1));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    // We only have one player, so we should not have sent any packets.
    sent_packets.assert_count::<SetEquipmentS2c>(0);
}

#[test]
fn test_multiple_entities() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        layer,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let zombie_bundle = ZombieEntityBundle {
        layer: EntityLayerId(layer),
        ..Default::default()
    };

    let zombie = app.world_mut().spawn(zombie_bundle).id();

    app.update();

    let mut equipment = app
        .world_mut()
        .get_mut::<Equipment>(zombie)
        .expect("could not get entity equipment");

    equipment.set_chest(ItemStack::new(ItemKind::DiamondChestplate, 1));
    equipment.set_head(ItemStack::new(ItemKind::DiamondHelmet, 1));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<SetEquipmentS2c>(1);

    helper.clear_received();

    let mut equipment = app
        .world_mut()
        .get_mut::<Equipment>(zombie)
        .expect("could not get entity equipment");

    // Set the zombie's equipment to the same items
    equipment.set_chest(ItemStack::new(ItemKind::DiamondChestplate, 1));
    equipment.set_head(ItemStack::new(ItemKind::DiamondHelmet, 1));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<SetEquipmentS2c>(0);
}

#[test]
fn test_update_on_load_entity() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        layer,
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let zombie_bundle = ZombieEntityBundle {
        layer: EntityLayerId(layer),
        position: Position::new(DVec3::new(1000.0, 0.0, 1000.0)),
        ..Default::default()
    };

    let zombie = app
        .world_mut()
        .spawn(zombie_bundle)
        .insert(Equipment::default())
        .id();

    app.update();

    let mut equipment = app
        .world_mut()
        .get_mut::<Equipment>(zombie)
        .expect("could not get entity equipment");

    equipment.set_chest(ItemStack::new(ItemKind::DiamondChestplate, 1));
    equipment.set_head(ItemStack::new(ItemKind::DiamondHelmet, 1));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    // The zombie is not in range of the player
    sent_packets.assert_count::<SetEquipmentS2c>(0);

    // Move the player to the zombie
    let mut player_pos = app
        .world_mut()
        .get_mut::<Position>(client)
        .expect("could not get player position");

    player_pos.0 = DVec3::new(1000.0, 0.0, 1000.0);

    // 1 tick for the tp, 1 tick for loading the entity (i think)
    app.update();
    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    // Once the player is in range, we send the equipment update
    sent_packets.assert_count::<SetEquipmentS2c>(1);
}

#[test]
fn test_skip_update_for_empty_equipment() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        layer,
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let zombie_bundle = ZombieEntityBundle {
        layer: EntityLayerId(layer),
        position: Position::new(DVec3::new(1000.0, 0.0, 1000.0)),
        ..Default::default()
    };

    app.world_mut()
        .spawn(zombie_bundle)
        .insert(Equipment::default());

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    // The zombie is not in range of the player
    sent_packets.assert_count::<SetEquipmentS2c>(0);

    // Move the player to the zombie
    let mut player_pos = app
        .world_mut()
        .get_mut::<Position>(client)
        .expect("could not get player position");

    player_pos.0 = DVec3::new(1000.0, 0.0, 1000.0);

    // 1 tick for the tp, 1 tick for loading the entity (i think)
    app.update();
    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    // We skip the packet, because the equipment is empty
    sent_packets.assert_count::<SetEquipmentS2c>(0);
}

#[test]
fn test_ensure_living_entities_only() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        layer,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let zombie_bundle = ZombieEntityBundle {
        layer: EntityLayerId(layer),
        ..Default::default()
    };

    let armor_stand_bundle = ArmorStandEntityBundle {
        layer: EntityLayerId(layer),
        ..Default::default()
    };

    let item_bundle = ItemEntityBundle {
        layer: EntityLayerId(layer),
        ..Default::default()
    };

    let zombie = app.world_mut().spawn(zombie_bundle).id();

    let armor_stand = app.world_mut().spawn(armor_stand_bundle).id();

    let item = app.world_mut().spawn(item_bundle).id();

    app.update();

    let zombie_equipment = app.world_mut().get_mut::<Equipment>(zombie);
    assert!(zombie_equipment.is_some());

    let armor_stand_equipment = app.world_mut().get_mut::<Equipment>(armor_stand);
    assert!(armor_stand_equipment.is_some());

    let item_equipment = app.world_mut().get_mut::<Equipment>(item);
    assert!(item_equipment.is_none());
}

#[test]
fn test_inventory_sync_from_equipment() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    app.world_mut()
        .entity_mut(client)
        .insert(EquipmentInventorySync);

    let mut player_equipment = app
        .world_mut()
        .get_mut::<Equipment>(client)
        .expect("could not get player equipment");

    player_equipment.set_chest(ItemStack::new(ItemKind::DiamondChestplate, 1));

    app.update();

    let player_inventory = app
        .world()
        .get::<Inventory>(client)
        .expect("could not get player equipment");

    let player_equipment = app
        .world()
        .get::<Equipment>(client)
        .expect("could not get player equipment");

    // The inventory should have been updated
    // after the equipment change
    assert_eq!(
        *player_inventory.slot(PlayerInventory::SLOT_CHEST),
        ItemStack::new(ItemKind::DiamondChestplate, 1)
    );

    assert_eq!(
        *player_equipment.chest(),
        ItemStack::new(ItemKind::DiamondChestplate, 1)
    );
}

#[test]
fn test_equipment_sync_from_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    app.world_mut()
        .entity_mut(client)
        .insert(EquipmentInventorySync);

    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not get player equipment");

    player_inventory.set_slot(
        PlayerInventory::SLOT_CHEST,
        ItemStack::new(ItemKind::DiamondChestplate, 1),
    );

    app.update();

    let player_inventory = app
        .world()
        .get::<Inventory>(client)
        .expect("could not get player equipment");

    let player_equipment = app
        .world()
        .get::<Equipment>(client)
        .expect("could not get player equipment");

    // The equipment should have been updated
    // after the inventory change
    assert_eq!(
        *player_inventory.slot(PlayerInventory::SLOT_CHEST),
        ItemStack::new(ItemKind::DiamondChestplate, 1)
    );

    assert_eq!(
        *player_equipment.chest(),
        ItemStack::new(ItemKind::DiamondChestplate, 1)
    );
}

#[test]
fn test_equipment_priority_over_inventory() {
    let ScenarioSingleClient {
        mut app, client, ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();

    app.world_mut()
        .entity_mut(client)
        .insert(EquipmentInventorySync);

    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not get player equipment");

    // Set the slot in the inventory as well as in the equipment in the same tick

    player_inventory.set_slot(
        PlayerInventory::SLOT_CHEST,
        ItemStack::new(ItemKind::DiamondChestplate, 1),
    );

    let mut player_equipment = app
        .world_mut()
        .get_mut::<Equipment>(client)
        .expect("could not get player equipment");

    player_equipment.set_chest(ItemStack::new(ItemKind::GoldenChestplate, 1));

    app.update();

    // The equipment change should have priority, the inventory change is ignored

    let player_inventory = app
        .world()
        .get::<Inventory>(client)
        .expect("could not get player equipment");

    let player_equipment = app
        .world()
        .get::<Equipment>(client)
        .expect("could not get player equipment");

    assert_eq!(
        *player_inventory.slot(PlayerInventory::SLOT_CHEST),
        ItemStack::new(ItemKind::GoldenChestplate, 1)
    );

    assert_eq!(
        *player_equipment.chest(),
        ItemStack::new(ItemKind::GoldenChestplate, 1)
    );
}

#[test]
fn test_equipment_change_from_player() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    app.world_mut()
        .entity_mut(client)
        .insert(EquipmentInventorySync);

    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not get player equipment");

    player_inventory.set_slot(36, ItemStack::new(ItemKind::DiamondChestplate, 1));
    app.update();
    helper.clear_received();

    let state_id = app
        .world()
        .get::<ClientInventoryState>(client)
        .expect("could not get player equipment")
        .state_id();

    app.update();

    helper.send(&ContainerClickC2s {
        window_id: 0.into(),
        button: 0,
        mode: ClickMode::Hotbar,
        state_id: state_id.0.into(),
        slot_idx: 36,
        slot_changes: vec![
            HashedSlotChange {
                idx: 36,
                stack: HashedItemStack::EMPTY,
            },
            HashedSlotChange {
                idx: PlayerInventory::SLOT_CHEST as i16,
                stack: HashedItemStack::new(ItemKind::DiamondChestplate, 1),
            },
        ]
        .into(),
        carried_item: HashedItemStack::EMPTY,
    });

    app.update();
    app.update();

    let player_inventory = app
        .world()
        .get::<Inventory>(client)
        .expect("could not get player equipment");

    let player_equipment = app
        .world()
        .get::<Equipment>(client)
        .expect("could not get player equipment");

    assert_eq!(
        player_inventory.slot(PlayerInventory::SLOT_CHEST),
        &ItemStack::new(ItemKind::DiamondChestplate, 1)
    );

    assert_eq!(player_inventory.slot(36), &ItemStack::EMPTY);

    assert_eq!(
        player_equipment.chest(),
        &ItemStack::new(ItemKind::DiamondChestplate, 1)
    );
}

#[test]
fn test_held_item_change_from_client() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    app.world_mut()
        .entity_mut(client)
        .insert(EquipmentInventorySync);

    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not get player equipment");

    player_inventory.set_slot(36, ItemStack::new(ItemKind::DiamondSword, 1));
    player_inventory.set_slot(37, ItemStack::new(ItemKind::IronSword, 1));

    app.update();

    let player_equipment = app
        .world()
        .get::<Equipment>(client)
        .expect("could not get player equipment");

    assert_eq!(
        player_equipment.main_hand(),
        &ItemStack::new(ItemKind::DiamondSword, 1)
    );

    // Change the held item from the client
    helper.send(&SetCarriedItemC2s { slot: 1 });

    app.update(); // handle change slot
    app.update(); // handle change equipment

    let player_equipment = app
        .world()
        .get::<Equipment>(client)
        .expect("could not get player equipment");

    assert_eq!(
        player_equipment.main_hand(),
        &ItemStack::new(ItemKind::IronSword, 1)
    );
}
