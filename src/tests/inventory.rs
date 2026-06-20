use std::borrow::Cow;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_inventory::player_inventory::PlayerInventory;
use chunkedge_item::ItemComponent;
use chunkedge_server::protocol::IntoTextComponent;

use crate::inventory::{
    convert_to_player_slot_id, ClickMode, ClientInventoryState, CursorItem, DropItemStackEvent,
    HeldItem, Inventory, InventoryKind, OpenInventory, SlotChange,
};
use crate::protocol::packets::play::{
    ContainerClickC2s, ContainerCloseS2c, ContainerSetContentS2c, ContainerSetSlotS2c,
    OpenScreenS2c, SetCarriedItemC2s, SetCreativeModeSlotC2s,
};
use crate::protocol::VarInt;
use crate::testing::{PacketFrames, ScenarioSingleClient};
use crate::{GameMode, ItemKind, ItemStack};

#[test]
fn test_should_open_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory = Inventory::new(InventoryKind::Generic3x3);
    let inventory_ent = app.world_mut().spawn(inventory).id();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Open the inventory.
    let open_inventory = OpenInventory::new(inventory_ent);
    app.world_mut()
        .get_entity_mut(client)
        .expect("could not find client")
        .insert(open_inventory);

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    sent_packets.assert_count::<OpenScreenS2c>(1);
    sent_packets.assert_count::<ContainerSetContentS2c>(1);
    sent_packets.assert_order::<(OpenScreenS2c, ContainerSetContentS2c)>();
}

#[test]
fn test_should_close_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory = Inventory::new(InventoryKind::Generic3x3);
    let inventory_ent = app.world_mut().spawn(inventory).id();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Open the inventory.
    let open_inventory = OpenInventory::new(inventory_ent);
    app.world_mut()
        .get_entity_mut(client)
        .expect("could not find client")
        .insert(open_inventory);

    app.update();
    helper.clear_received();

    // Close the inventory.
    app.world_mut()
        .get_entity_mut(client)
        .expect("could not find client")
        .remove::<OpenInventory>();

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    sent_packets.assert_count::<ContainerCloseS2c>(1);
}

#[test]
fn test_should_remove_invalid_open_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory = Inventory::new(InventoryKind::Generic3x3);
    let inventory_ent = app.world_mut().spawn(inventory).id();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Open the inventory.
    let open_inventory = OpenInventory::new(inventory_ent);
    app.world_mut()
        .get_entity_mut(client)
        .expect("could not find client")
        .insert(open_inventory);

    app.update();
    helper.clear_received();

    // Remove the inventory.
    app.world_mut().despawn(inventory_ent);

    app.update();

    // Make assertions
    assert!(app.world_mut().get::<OpenInventory>(client).is_none());

    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<ContainerCloseS2c>(1);
}

#[test]
fn test_should_modify_player_inventory_click_slot() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");
    inventory.set_slot(
        20,
        ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    // Make the client click the slot and pick up the item.
    let state_id = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .unwrap()
        .state_id();

    helper.send(&ContainerClickC2s {
        window_id: VarInt(0),
        button: 0,
        mode: ClickMode::Click,
        state_id: VarInt(state_id.0),
        slot_idx: 20,
        slot_changes: Cow::Borrowed(
            [SlotChange {
                idx: 20,
                stack: ItemStack::EMPTY,
            }
            .into()]
            .as_slice(),
        ),
        carried_item: ItemStack::new(ItemKind::Diamond, 2)
            .with_components(vec![
                ItemComponent::CustomName("Custom Diamond".into_text_component()),
                ItemComponent::Lore(vec![
                    "Lore Line 1.".into_text_component(),
                    "Lore line 2.".into_text_component(),
                ]),
            ])
            .into(),
    });

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    // because the inventory was changed as a result of the client's click, the
    // server should not send any packets to the client because the client
    // already knows about the change.
    sent_packets.assert_count::<ContainerSetContentS2c>(0);
    sent_packets.assert_count::<ContainerSetSlotS2c>(0);

    let inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find inventory for client");

    assert_eq!(inventory.slot(20), &ItemStack::EMPTY);

    let cursor_item = app
        .world_mut()
        .get::<CursorItem>(client)
        .expect("could not find client");

    assert_eq!(
        cursor_item.0,
        ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );
}

#[test]
fn test_should_allow_non_modifying_inventory_clicks() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");
    inventory.set_slot(20, ItemStack::new(ItemKind::Diamond, 2));

    // Make the client click the slot and pick up the item.
    let state_id = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .unwrap()
        .state_id();
    // Used keyboard to "click" on one of the slots, but both the hovered slot and
    // the target hotbar slot are empty
    helper.send(&ContainerClickC2s {
        window_id: 0.into(),
        button: 0,
        mode: ClickMode::Hotbar,
        state_id: VarInt(state_id.0),
        slot_idx: 0,
        slot_changes: vec![].into(),
        carried_item: ItemStack::new(ItemKind::Air, 0).into(),
    });
    // Clicked on a real slot, but the slot is empty
    helper.send(&ContainerClickC2s {
        window_id: 0.into(),
        button: 0,
        mode: ClickMode::Click,
        state_id: VarInt(state_id.0),
        slot_idx: 1,
        slot_changes: vec![].into(),
        carried_item: ItemStack::new(ItemKind::Air, 0).into(),
    });
    // Clicked in the margin area of an inventory (in the main ui but not in one of
    // the slots)
    helper.send(&ContainerClickC2s {
        window_id: 0.into(),
        button: 0,
        mode: ClickMode::Click,
        state_id: VarInt(state_id.0),
        slot_idx: -1,
        slot_changes: vec![].into(),
        carried_item: ItemStack::new(ItemKind::Air, 0).into(),
    });
    // Clicked outside the user interface without holding an item (this is a drop
    // key mode from the client for some reason)
    helper.send(&ContainerClickC2s {
        window_id: 0.into(),
        button: 0,
        mode: ClickMode::DropKey,
        state_id: VarInt(state_id.0),
        slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
        slot_changes: vec![].into(),
        carried_item: ItemStack::new(ItemKind::Air, 0).into(),
    });

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    // The user intereacted with the inventory themselves, and should not get a
    // resync
    sent_packets.assert_count::<ContainerSetContentS2c>(0);
}

#[test]
fn test_should_keep_cursor_for_non_modifying_armor_click() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    app.update();
    helper.clear_received();

    let stack = ItemStack::new(ItemKind::IronSword, 1).with_components(vec![
        ItemComponent::Unbreakable,
        ItemComponent::CustomName("Custom Item Name".into_text_component()),
    ]);

    app.world_mut().get_mut::<CursorItem>(client).unwrap().0 = stack.clone();

    let state_id = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .unwrap()
        .state_id();

    helper.send(&ContainerClickC2s {
        window_id: VarInt(0),
        button: 0,
        mode: ClickMode::Click,
        state_id: VarInt(state_id.0),
        slot_idx: 6,
        slot_changes: vec![].into(),
        carried_item: stack.clone().into(),
    });

    app.update();

    let cursor_item = app
        .world_mut()
        .get::<CursorItem>(client)
        .expect("could not find client");
    assert_eq!(cursor_item.0, stack);

    let inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find inventory for client");
    assert_eq!(inventory.slot(6), &ItemStack::EMPTY);

    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<ContainerSetContentS2c>(0);
    sent_packets.assert_count::<ContainerSetSlotS2c>(0);
}

#[test]
fn test_should_modify_player_inventory_server_side() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");
    inventory.set_slot(20, ItemStack::new(ItemKind::Diamond, 2));

    app.update();
    helper.clear_received();

    // Modify the inventory.
    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");
    inventory.set_slot(21, ItemStack::new(ItemKind::IronIngot, 1));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    // because the inventory was modified server side, the client needs to be
    // updated with the change.
    sent_packets.assert_count::<ContainerSetSlotS2c>(1);
}

#[test]
fn test_should_sync_entire_player_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");
    inventory.changed = u64::MAX;

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<ContainerSetContentS2c>(1);
}

fn set_up_open_inventory(app: &mut App, client_ent: Entity) -> Entity {
    let inventory = Inventory::new(InventoryKind::Generic9x3);
    let inventory_ent = app.world_mut().spawn(inventory).id();

    // Open the inventory.
    let open_inventory = OpenInventory::new(inventory_ent);
    app.world_mut()
        .get_entity_mut(client_ent)
        .expect("could not find client")
        .insert(open_inventory);

    inventory_ent
}

fn describe_packet_ids(sent_packets: &PacketFrames) -> String {
    sent_packets
        .0
        .iter()
        .map(|frame| format!("Unknown({:?})", frame.id))
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn test_should_modify_open_inventory_click_slot() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory_ent = set_up_open_inventory(&mut app, client);

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(inventory_ent)
        .expect("could not find inventory for client");

    inventory.set_slot(
        20,
        ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Make the client click the slot and pick up the item.
    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let state_id = inv_state.state_id();
    let window_id = inv_state.window_id();
    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id.0),
        slot_idx: 20,
        button: 0,
        mode: ClickMode::Click,
        slot_changes: vec![SlotChange {
            idx: 20,
            stack: ItemStack::EMPTY,
        }
        .into()]
        .into(),
        carried_item: ItemStack::new(ItemKind::Diamond, 2)
            .with_components(vec![
                ItemComponent::CustomName("Custom Diamond".into_text_component()),
                ItemComponent::Lore(vec![
                    "Lore Line 1.".into_text_component(),
                    "Lore line 2.".into_text_component(),
                ]),
            ])
            .into(),
    });

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    // because the inventory was modified as a result of the client's click, the
    // server should not send any packets to the client because the client
    // already knows about the change.
    sent_packets.assert_count::<ContainerSetContentS2c>(0);
    sent_packets.assert_count::<ContainerSetSlotS2c>(0);

    let inventory = app
        .world_mut()
        .get::<Inventory>(inventory_ent)
        .expect("could not find inventory");

    assert_eq!(inventory.slot(20), &ItemStack::EMPTY);

    let cursor_item = app
        .world_mut()
        .get::<CursorItem>(client)
        .expect("could not find client");

    assert_eq!(
        cursor_item.0,
        ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );
}

#[test]
fn test_sync_inventory_change_made_from_chunkedge_while_inventory_is_opened() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Open a inventory for the player. (27 slots)
    set_up_open_inventory(&mut app, client);
    app.update();
    helper.clear_received();

    // Modify the player's inventory.
    let client_inv_state = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .expect("could not find client inventory state");

    let inv_state_window_id = client_inv_state.window_id();
    let inv_state_state_id = client_inv_state.state_id();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");

    inventory.set_slot(9, ItemStack::new(ItemKind::Diamond, 2));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    let received = sent_packets.0[0].decode::<ContainerSetSlotS2c>().unwrap();

    assert_eq!(received.window_id, inv_state_window_id);

    assert_eq!(received.slot_idx, 27);

    assert_eq!(
        received.slot_data,
        Cow::Borrowed(&ItemStack::new(ItemKind::Diamond, 2))
    );

    assert_eq!(received.state_id, VarInt(inv_state_state_id.0));
}

#[test]
fn test_prevent_modify_open_inventory_click_slot_readonly_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // The open inventory is readonly, the client can not interact with it.
    let inventory_ent = set_up_open_inventory(&mut app, client);

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(inventory_ent)
        .expect("could not find inventory for client");

    inventory.readonly = true;
    inventory.set_slot(
        20,
        ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Make the client click the slot and pick up the item.
    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let state_id = inv_state.state_id();
    let window_id = inv_state.window_id();

    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id.0),
        slot_idx: 20,
        button: 0,
        // If the inventory is readonly, this should actually not be possible,
        // as you cant even select an item (so its on your cursor),
        // this is also why 2 resyncs are sent, see below.
        mode: ClickMode::Click,
        slot_changes: vec![SlotChange {
            idx: 20,
            stack: ItemStack::EMPTY,
        }
        .into()]
        .into(),
        carried_item: ItemStack::new(ItemKind::Diamond, 2)
            .with_components(vec![
                ItemComponent::CustomName("Custom Diamond".into_text_component()),
                ItemComponent::Lore(vec![
                    "Lore Line 1.".into_text_component(),
                    "Lore line 2.".into_text_component(),
                ]),
            ])
            .into(),
    });

    app.update();

    let sent_packets = helper.collect_received();

    // because the inventory is readonly, we need to resync the client's inventory.
    // 2 resync packets are sent, see above.
    sent_packets.assert_count::<ContainerSetContentS2c>(2);
    sent_packets.assert_count::<ContainerSetSlotS2c>(0);

    // Make assertions
    let inventory = app
        .world_mut()
        .get::<Inventory>(inventory_ent)
        .expect("could not find inventory");
    // Inventory is read-only, the item is not being moved
    assert_eq!(
        inventory.slot(20),
        &ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );
    let cursor_item = app
        .world_mut()
        .get::<CursorItem>(client)
        .expect("could not find client");
    // Inventory is read-only, items can not be picked up with the cursor
    assert_eq!(cursor_item.0, ItemStack::EMPTY);
}

#[test]
fn test_should_modify_open_inventory_server_side() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory_ent = set_up_open_inventory(&mut app, client);

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // Modify the inventory.
    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(inventory_ent)
        .expect("could not find inventory for client");
    inventory.set_slot(5, ItemStack::new(ItemKind::IronIngot, 1));

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();

    // because the inventory was modified server side, the client needs to be
    // updated with the change.
    sent_packets.assert_count::<ContainerSetSlotS2c>(1);

    let inventory = app
        .world_mut()
        .get::<Inventory>(inventory_ent)
        .expect("could not find inventory for client");

    assert_eq!(inventory.slot(5), &ItemStack::new(ItemKind::IronIngot, 1));
}

#[test]
fn test_hotbar_item_swap_container() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();

    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");

    // 36 is the first hotbar slot
    player_inventory.set_slot(
        36,
        ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    let open_inv_ent = set_up_open_inventory(&mut app, client);

    let mut open_inventory = app
        .world_mut()
        .get_mut::<Inventory>(open_inv_ent)
        .expect("could not find inventory for client");

    open_inventory.set_slot(
        0,
        ItemStack::new(ItemKind::IronIngot, 10).with_components(vec![
            ItemComponent::CustomName("Custom Iron".into_text_component()),
            ItemComponent::Lore(vec![
                "Other Lore Line 1.".into_text_component(),
                "Other Lore Line 2.".into_text_component(),
            ]),
        ]),
    );

    // This update makes sure we have the items in the inventory by the time the
    // client wants to update these
    app.update();
    helper.clear_received();
    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let state_id = inv_state.state_id();
    let window_id = inv_state.window_id();

    // The player hovers over the iron ingots in the open inventory, and tries
    // to move them to their own (via pressing 1), which should swap the iron
    // for the diamonds.
    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id.0),
        slot_idx: 0,
        button: 0, // hotbar slot starting at 0
        mode: ClickMode::Hotbar,
        slot_changes: vec![
            // First SlotChange is the item is the slot in the player's hotbar.
            // target slot.
            SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                // 54 is the players hotbar slot 1, when the 9x3 inventory is opnened.
                idx: 54,
                stack: ItemStack::new(ItemKind::IronIngot, 10).with_components(vec![
                    ItemComponent::CustomName("Custom Iron".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Other Lore Line 1.".into_text_component(),
                        "Other Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            // The second one is the slot in the open inventory, after the ClickSlot action
            // source slot.
        ]
        .into(),
        carried_item: ItemStack::EMPTY.into(),
    });

    app.update();

    let sent_packets = helper.collect_received();

    // No resyncs because the client was in sync and sent us the updates
    sent_packets.assert_count::<ContainerSetContentS2c>(0);

    // Make assertions
    let player_inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find client");

    // Swapped items successfully
    assert_eq!(
        player_inventory.slot(36),
        &ItemStack::new(ItemKind::IronIngot, 10).with_components(vec![
            ItemComponent::CustomName("Custom Iron".into_text_component()),
            ItemComponent::Lore(vec![
                "Other Lore Line 1.".into_text_component(),
                "Other Lore Line 2.".into_text_component(),
            ]),
        ])
    );

    let open_inventory = app
        .world_mut()
        .get::<Inventory>(open_inv_ent)
        .expect("could not find inventory");

    assert_eq!(
        open_inventory.slot(0),
        &ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );
}

#[test]
fn test_prevent_hotbar_item_click_container_readonly_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // player inventory is not read-only
    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");

    // 36 is the first hotbar slot
    player_inventory.set_slot(
        36,
        ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    let open_inv_ent = set_up_open_inventory(&mut app, client);

    let mut open_inventory = app
        .world_mut()
        .get_mut::<Inventory>(open_inv_ent)
        .expect("could not find inventory for client");

    // Open inventory is read-only
    open_inventory.readonly = true;
    open_inventory.set_slot(
        0,
        ItemStack::new(ItemKind::IronIngot, 10).with_components(vec![
            ItemComponent::CustomName("Custom Iron".into_text_component()),
            ItemComponent::Lore(vec![
                "Other Lore Line 1.".into_text_component(),
                "Other Lore Line 2.".into_text_component(),
            ]),
        ]),
    );

    // This update makes sure we have the items in the inventory by the time the
    // client wants to update these
    app.update();
    helper.clear_received();

    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let state_id = inv_state.state_id();
    let window_id = inv_state.window_id();

    // The player hovers over the iron ingots in the open inventory, and tries
    // to move them to their own (via pressing 1), which should swap the iron
    // for the diamonds. However the opened inventory is read-only, so nothing
    // should happen.
    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id.0),
        slot_idx: 0,
        button: 0, // hotbar slot starting at 0
        mode: ClickMode::Hotbar,
        slot_changes: vec![
            // First SlotChange is the item is the slot in the player's hotbar.
            // target slot.
            SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            // The second one is the slot in the open inventory, after the ClickSlot action
            // source slot.
            SlotChange {
                // 54 is the players hotbar slot 1, when the 9x3 inventory is opnened.
                idx: 54,
                stack: ItemStack::new(ItemKind::IronIngot, 10).with_components(vec![
                    ItemComponent::CustomName("Custom Iron".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Other Lore Line 1.".into_text_component(),
                        "Other Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
        ]
        .into(),
        carried_item: ItemStack::EMPTY.into(),
    });

    app.update();

    let sent_packets = helper.collect_received();

    // 1 resync for each inventory
    sent_packets.assert_count::<ContainerSetContentS2c>(2);

    // Make assertions
    let player_inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find client");

    // Opened inventory is read-only, the items are not swapped.
    assert_eq!(
        player_inventory.slot(36),
        &ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );

    let open_inventory = app
        .world_mut()
        .get::<Inventory>(open_inv_ent)
        .expect("could not find inventory");

    // Opened inventory is read-only, the items are not swapped.
    assert_eq!(
        open_inventory.slot(0),
        &ItemStack::new(ItemKind::IronIngot, 10).with_components(vec![
            ItemComponent::CustomName("Custom Iron".into_text_component()),
            ItemComponent::Lore(vec![
                "Other Lore Line 1.".into_text_component(),
                "Other Lore Line 2.".into_text_component(),
            ]),
        ])
    );
}

#[test]
fn test_still_allow_hotbar_item_click_in_own_inventory_if_container_readonly_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // player inventory is not read-only
    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");

    // 36 is the first hotbar slot
    player_inventory.set_slot(
        36,
        ItemStack::new(ItemKind::Diamond, 10).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    let open_inv_ent = set_up_open_inventory(&mut app, client);

    let mut open_inventory = app
        .world_mut()
        .get_mut::<Inventory>(open_inv_ent)
        .expect("could not find inventory for client");

    // Open inventory is read-only
    open_inventory.readonly = true;

    // This update makes sure we have the items in the inventory by the time the
    // client wants to update these
    app.update();
    helper.clear_received();

    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let state_id = inv_state.state_id();
    let window_id = inv_state.window_id();

    // The player's inventory is not readonly, so the player should still be
    // able to move items from the hotbar to other parts of the inventory even
    // if the other inventory is still open.
    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id.0),
        slot_idx: 27,
        button: 0, // hotbar slot starting at 0
        mode: ClickMode::Hotbar,
        slot_changes: vec![
            SlotChange {
                idx: 27,
                stack: ItemStack::new(ItemKind::Diamond, 10).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 54,
                stack: ItemStack::EMPTY,
            }
            .into(),
        ]
        .into(),
        carried_item: ItemStack::EMPTY.into(),
    });

    app.update();
    // Make assertions
    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<ContainerSetContentS2c>(2);

    let player_inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find client");

    // Items swapped successfully, as player item is not read-only
    assert_eq!(player_inventory.slot(36), &ItemStack::EMPTY);
    assert_eq!(
        player_inventory.slot(9),
        &ItemStack::new(ItemKind::Diamond, 10).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );
}

#[test]
fn test_prevent_shift_item_click_container_readonly_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    // player inventory is not read-only
    let mut player_inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory for client");

    player_inventory.set_slot(
        9,
        ItemStack::new(ItemKind::Diamond, 64).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ]),
    );

    let open_inv_ent = set_up_open_inventory(&mut app, client);

    let mut open_inventory = app
        .world_mut()
        .get_mut::<Inventory>(open_inv_ent)
        .expect("could not find inventory for client");

    // Open inventory is read-only
    open_inventory.readonly = true;

    // This update makes sure we have the items in the inventory by the time the
    // client wants to update these
    app.update();
    helper.clear_received();

    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let state_id = inv_state.state_id();
    let window_id = inv_state.window_id();

    // The player tries to Shift-click transfer the stack of diamonds into
    // the open container
    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id.0),
        slot_idx: 27,
        button: 0, // hotbar slot starting at 0
        mode: ClickMode::ShiftClick,
        slot_changes: vec![
            // target
            SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 64).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            // source
            SlotChange {
                idx: 27,
                stack: ItemStack::EMPTY,
            }
            .into(),
        ]
        .into(),
        carried_item: ItemStack::EMPTY.into(),
    });

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    // 1 resync per inventory
    sent_packets.assert_count::<ContainerSetContentS2c>(2);

    let player_inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find client");

    assert_eq!(
        player_inventory.slot(9),
        &ItemStack::new(ItemKind::Diamond, 64).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore line 2.".into_text_component(),
            ]),
        ])
    );

    let open_inventory = app
        .world_mut()
        .get::<Inventory>(open_inv_ent)
        .expect("could not find inventory");

    assert_eq!(open_inventory.slot(0), &ItemStack::EMPTY);
}

#[test]
fn test_should_sync_entire_open_inventory() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory_ent = set_up_open_inventory(&mut app, client);

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(inventory_ent)
        .expect("could not find inventory");
    inventory.changed = u64::MAX;

    app.update();

    // Make assertions
    let sent_packets = helper.collect_received();
    sent_packets.assert_count::<ContainerSetContentS2c>(1);
}

#[test]
fn test_set_creative_mode_slot_handling() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let mut game_mode = app
        .world_mut()
        .get_mut::<GameMode>(client)
        .expect("could not find client");
    *game_mode.as_mut() = GameMode::Creative;

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    helper.send(&SetCreativeModeSlotC2s {
        slot: 36,
        clicked_item: ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Creative Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Creative Lore Line 1.".into_text_component(),
                "Creative Lore line 2.".into_text_component(),
            ]),
        ]),
    });

    app.update();

    // Make assertions
    let inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find inventory for client");

    assert_eq!(
        inventory.slot(36),
        &ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Creative Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Creative Lore Line 1.".into_text_component(),
                "Creative Lore line 2.".into_text_component(),
            ]),
        ])
    );
}

#[test]
fn test_ignore_set_creative_mode_slot_if_not_creative() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let mut game_mode = app
        .world_mut()
        .get_mut::<GameMode>(client)
        .expect("could not find client");
    *game_mode.as_mut() = GameMode::Survival;

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    helper.send(&SetCreativeModeSlotC2s {
        slot: 36,
        clicked_item: ItemStack::new(ItemKind::Diamond, 2).with_components(vec![
            ItemComponent::CustomName("Creative Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Creative Lore Line 1.".into_text_component(),
                "Creative Lore line 2.".into_text_component(),
            ]),
        ]),
    });

    app.update();

    // Make assertions
    let inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find inventory for client");
    assert_eq!(inventory.slot(36), &ItemStack::EMPTY);
}

#[test]
fn test_window_id_increments() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    let inventory = Inventory::new(InventoryKind::Generic9x3);
    let inventory_ent = app.world_mut().spawn(inventory).id();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    for _ in 0..3 {
        let open_inventory = OpenInventory::new(inventory_ent);
        app.world_mut()
            .get_entity_mut(client)
            .expect("could not find client")
            .insert(open_inventory);

        app.update();

        app.world_mut()
            .get_entity_mut(client)
            .expect("could not find client")
            .remove::<OpenInventory>();

        app.update();
    }

    // Make assertions
    let inv_state = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .expect("could not find client");
    assert_eq!(inv_state.window_id(), VarInt(3));
}

#[test]
fn test_should_handle_set_held_item() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    helper.send(&SetCarriedItemC2s { slot: 4 });

    app.update();

    // Make assertions
    let held = app
        .world_mut()
        .get::<HeldItem>(client)
        .expect("could not find client");

    assert_eq!(held.slot(), 40);
}

#[test]
fn should_not_increment_state_id_on_cursor_item_change() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let inv_state = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .expect("could not find client");
    let expected_state_id = inv_state.state_id().0;

    let mut cursor_item = app.world_mut().get_mut::<CursorItem>(client).unwrap();
    cursor_item.0 = ItemStack::new(ItemKind::Diamond, 2);

    app.update();

    // Make assertions
    let inv_state = app
        .world_mut()
        .get::<ClientInventoryState>(client)
        .expect("could not find client");
    assert_eq!(
        inv_state.state_id().0,
        expected_state_id,
        "state id should not have changed"
    );
}

#[test]
fn should_send_cursor_item_change_when_modified_on_the_server() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();
    helper.clear_received();

    let mut cursor_item = app.world_mut().get_mut::<CursorItem>(client).unwrap();
    cursor_item.0 = ItemStack::new(ItemKind::Diamond, 2);

    app.update();

    let sent_packets = helper.collect_received();

    sent_packets.assert_count::<ContainerSetSlotS2c>(1);
}

mod dropping_items {
    use super::*;
    use crate::inventory::{convert_to_player_slot_id, PlayerAction};
    use crate::protocol::packets::play::PlayerActionC2s;
    use crate::{BlockPos, Direction};

    #[test]
    fn should_drop_item_player_action() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");
        inventory.set_slot(36, ItemStack::new(ItemKind::IronIngot, 3));

        helper.send(&PlayerActionC2s {
            action: PlayerAction::DropItem,
            position: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            sequence: VarInt(0),
        });

        app.update();

        // Make assertions
        let inventory = app
            .world_mut()
            .get::<Inventory>(client)
            .expect("could not find client");

        assert_eq!(inventory.slot(36), &ItemStack::new(ItemKind::IronIngot, 2));

        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(events[0].from_slot, Some(36));
        assert_eq!(events[0].stack, ItemStack::new(ItemKind::IronIngot, 1));

        let sent_packets = helper.collect_received();

        sent_packets.assert_count::<ContainerSetSlotS2c>(0);
    }

    #[test]
    fn prevent_drop_item_player_action_readonly_inventory() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");
        inventory.readonly = true;
        inventory.set_slot(36, ItemStack::new(ItemKind::IronIngot, 3));

        helper.send(&PlayerActionC2s {
            action: PlayerAction::DropItem,
            position: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            sequence: VarInt(0),
        });

        app.update();

        // Make assertions
        let inventory = app
            .world_mut()
            .get::<Inventory>(client)
            .expect("could not find client");

        assert_eq!(
            inventory.slot(36),
            // Inventory is read-only, item is not being dropped
            &ItemStack::new(ItemKind::IronIngot, 3)
        );

        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        // when the inventory is read-only we do not emit a drop event
        assert_eq!(events.len(), 0);

        let sent_packets = helper.collect_received();

        // we do need to update the player inventory so we dont desync
        sent_packets.assert_count::<ContainerSetSlotS2c>(1);
    }

    #[test]
    fn should_drop_item_stack_player_action() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");
        inventory.set_slot(36, ItemStack::new(ItemKind::IronIngot, 32));

        helper.send(&PlayerActionC2s {
            action: PlayerAction::DropAllItems,
            position: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            sequence: VarInt(0),
        });

        app.update();

        // Make assertions
        let held = app
            .world_mut()
            .get::<HeldItem>(client)
            .expect("could not find client");
        assert_eq!(held.slot(), 36);
        let inventory = app
            .world_mut()
            .get::<Inventory>(client)
            .expect("could not find inventory");
        assert_eq!(inventory.slot(36), &ItemStack::EMPTY);
        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");
        let events = events.iter_current_update_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(events[0].from_slot, Some(36));
        assert_eq!(events[0].stack, ItemStack::new(ItemKind::IronIngot, 32));
    }

    #[test]
    fn prevent_drop_item_stack_player_action_readonly_inventory() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");
        inventory.readonly = true;
        inventory.set_slot(36, ItemStack::new(ItemKind::IronIngot, 32));

        helper.send(&PlayerActionC2s {
            action: PlayerAction::DropAllItems,
            position: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            sequence: VarInt(0),
        });

        app.update();

        // Make assertions
        let held = app
            .world_mut()
            .get::<HeldItem>(client)
            .expect("could not find client");
        assert_eq!(held.slot(), 36);
        let inventory = app
            .world_mut()
            .get::<Inventory>(client)
            .expect("could not find inventory");
        // Inventory is read-only, item is not being dropped
        assert_eq!(inventory.slot(36), &ItemStack::new(ItemKind::IronIngot, 32));
        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");
        let events = events.iter_current_update_events().collect::<Vec<_>>();

        // when the inventory is read-only we do not emit a drop event
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn should_drop_item_stack_set_creative_mode_slot() {
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
            .insert(GameMode::Creative);

        helper.send(&SetCreativeModeSlotC2s {
            slot: -1,
            clicked_item: ItemStack::new(ItemKind::IronIngot, 32),
        });

        app.update();

        // Make assertions
        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events")
            .iter_current_update_events()
            .collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(events[0].from_slot, None);
        assert_eq!(events[0].stack, ItemStack::new(ItemKind::IronIngot, 32));
    }

    #[test]
    fn should_drop_item_stack_click_container_outside() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let mut cursor_item = app
            .world_mut()
            .get_mut::<CursorItem>(client)
            .expect("could not find client");

        cursor_item.0 = ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
            ItemComponent::CustomName("Droppable Iron".into_text_component()),
        ]);

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");
        let state_id = inv_state.state_id().0;

        helper.send(&ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(state_id),
            slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
            button: 0,
            mode: ClickMode::Click,
            slot_changes: vec![].into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let cursor_item = app
            .world_mut()
            .get::<CursorItem>(client)
            .expect("could not find client");

        assert_eq!(cursor_item.0, ItemStack::EMPTY);

        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(events[0].from_slot, None);
        assert_eq!(
            events[0].stack,
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Droppable Iron".into_text_component()),
            ])
        );
    }

    #[test]
    fn should_keep_carried_item_on_margin_click() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        app.update();
        helper.clear_received();

        let stack = ItemStack::new(ItemKind::IronSword, 1).with_components(vec![
            ItemComponent::Unbreakable,
            ItemComponent::CustomName("Custom Item Name".into_text_component()),
        ]);

        app.world_mut().get_mut::<CursorItem>(client).unwrap().0 = stack.clone();

        let state_id = app
            .world_mut()
            .get::<ClientInventoryState>(client)
            .expect("could not find client")
            .state_id()
            .0;

        helper.send(&ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(state_id),
            slot_idx: -1,
            button: 0,
            mode: ClickMode::Click,
            slot_changes: vec![].into(),
            carried_item: stack.clone().into(),
        });

        app.update();

        let cursor_item = app
            .world_mut()
            .get::<CursorItem>(client)
            .expect("could not find client");
        assert_eq!(cursor_item.0, stack);

        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events")
            .iter_current_update_events()
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 0);

        let sent_packets = helper.collect_received();
        sent_packets.assert_count::<ContainerSetContentS2c>(0);
        sent_packets.assert_count::<ContainerSetSlotS2c>(0);
    }

    #[test]
    fn should_drop_item_click_container_with_dropkey_single() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");

        let state_id = inv_state.state_id().0;

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        inventory.set_slot(
            40,
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ]),
        );

        helper.send(&ContainerClickC2s {
            window_id: VarInt(0),
            slot_idx: 40,
            button: 0,
            mode: ClickMode::DropKey,
            state_id: VarInt(state_id),
            slot_changes: vec![SlotChange {
                idx: 40,
                stack: ItemStack::new(ItemKind::IronIngot, 31).with_components(vec![
                    ItemComponent::CustomName("Custom Iron".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Other Lore Line 1.".into_text_component(),
                        "Other Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(events[0].from_slot, Some(40));
        assert_eq!(
            events[0].stack,
            ItemStack::new(ItemKind::IronIngot, 1).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ])
        );
    }

    #[test]
    fn prevent_drop_item_click_container_with_dropkey_single_readonly_inventory() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");

        let state_id = inv_state.state_id().0;

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        inventory.readonly = true;
        inventory.set_slot(
            40,
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ]),
        );

        helper.send(&ContainerClickC2s {
            window_id: VarInt(0),
            slot_idx: 40,
            button: 0,
            mode: ClickMode::DropKey,
            state_id: VarInt(state_id),
            slot_changes: vec![SlotChange {
                idx: 40,
                stack: ItemStack::new(ItemKind::IronIngot, 31).with_components(vec![
                    ItemComponent::CustomName("Custom Iron".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Other Lore Line 1.".into_text_component(),
                        "Other Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        assert_eq!(
            inventory.slot(40),
            // Inventory is read-only, item is not being dropped
            &ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ])
        );

        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        // when the inventory is read-only we do not emit a drop event
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn should_drop_item_stack_click_container_with_dropkey() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");

        let state_id = inv_state.state_id().0;

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        inventory.set_slot(
            40,
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ]),
        );

        helper.send(&ContainerClickC2s {
            window_id: VarInt(0),
            slot_idx: 40,
            button: 1, // pressing control
            mode: ClickMode::DropKey,
            state_id: VarInt(state_id),
            slot_changes: vec![SlotChange {
                idx: 40,
                stack: ItemStack::EMPTY,
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(events[0].from_slot, Some(40));
        assert_eq!(
            events[0].stack,
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ])
        );
    }

    #[test]
    fn prevent_drop_item_stack_click_container_with_dropkey_readonly_inventory() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();
        helper.clear_received();

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");

        let state_id = inv_state.state_id().0;

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        inventory.readonly = true;
        inventory.set_slot(
            40,
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ]),
        );

        helper.send(&ContainerClickC2s {
            window_id: VarInt(0),
            slot_idx: 40,
            button: 1, // pressing control
            mode: ClickMode::DropKey,
            state_id: VarInt(state_id),
            slot_changes: vec![SlotChange {
                idx: 40,
                stack: ItemStack::EMPTY,
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        assert_eq!(
            inventory.slot(40),
            // Inventory is read-only, item is not being dropped
            &ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ])
        );

        let events = app
            .world_mut()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        // when the inventory is read-only we do not emit a drop event
        assert_eq!(events.len(), 0);
    }

    /// The item should be dropped successfully, if the player has an inventory
    /// open and the slot id points to his inventory.
    #[test]
    fn should_drop_item_player_open_inventory_with_dropkey() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        inventory.set_slot(
            convert_to_player_slot_id(InventoryKind::Generic9x3, 50),
            ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ]),
        );

        let _inventory_ent = set_up_open_inventory(&mut app, client);

        app.update();

        helper.clear_received();

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");

        let state_id = inv_state.state_id().0;
        let window_id = inv_state.window_id();

        helper.send(&ContainerClickC2s {
            window_id,
            state_id: VarInt(state_id),
            slot_idx: 50, // not pressing control
            button: 0,
            mode: ClickMode::DropKey,
            slot_changes: vec![SlotChange {
                idx: 50,
                stack: ItemStack::new(ItemKind::IronIngot, 31).with_components(vec![
                    ItemComponent::CustomName("Custom Iron".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Other Lore Line 1.".into_text_component(),
                        "Other Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let events = app
            .world()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let player_inventory = app
            .world()
            .get::<Inventory>(client)
            .expect("could not find inventory");

        let events = events.iter_current_update_events().collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].client, client);
        assert_eq!(
            events[0].from_slot,
            Some(convert_to_player_slot_id(InventoryKind::Generic9x3, 50))
        );

        assert_eq!(
            events[0].stack,
            ItemStack::new(ItemKind::IronIngot, 1).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ])
        );

        // Also make sure that the player inventory was updated correctly.
        let expected_player_slot_id = convert_to_player_slot_id(InventoryKind::Generic9x3, 50);
        assert_eq!(
            player_inventory.slot(expected_player_slot_id),
            &ItemStack::new(ItemKind::IronIngot, 31).with_components(vec![
                ItemComponent::CustomName("Custom Iron".into_text_component()),
                ItemComponent::Lore(vec![
                    "Other Lore Line 1.".into_text_component(),
                    "Other Lore Line 2.".into_text_component(),
                ]),
            ])
        );
    }

    #[test]
    fn prevent_drop_item_player_open_inventory_with_dropkey_readonly_inventory() {
        let ScenarioSingleClient {
            mut app,
            client,
            mut helper,
            ..
        } = ScenarioSingleClient::new();

        // Process a tick to get past the "on join" logic.
        app.update();

        let mut inventory = app
            .world_mut()
            .get_mut::<Inventory>(client)
            .expect("could not find inventory");

        inventory.readonly = true;
        inventory.set_slot(
            convert_to_player_slot_id(InventoryKind::Generic9x3, 50),
            ItemStack::new(ItemKind::IronIngot, 32),
        );

        let _inventory_ent = set_up_open_inventory(&mut app, client);

        app.update();

        helper.clear_received();

        let inv_state = app
            .world_mut()
            .get_mut::<ClientInventoryState>(client)
            .expect("could not find client");

        let state_id = inv_state.state_id().0;
        let window_id = inv_state.window_id();

        helper.send(&ContainerClickC2s {
            window_id,
            state_id: VarInt(state_id),
            slot_idx: 50, // not pressing control
            button: 0,
            mode: ClickMode::DropKey,
            slot_changes: vec![SlotChange {
                idx: 50,
                stack: ItemStack::new(ItemKind::IronIngot, 31),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        });

        app.update();

        // Make assertions
        let events = app
            .world()
            .get_resource::<Events<DropItemStackEvent>>()
            .expect("expected drop item stack events");

        let player_inventory = app
            .world()
            .get::<Inventory>(client)
            .expect("could not find inventory");

        let events = events.iter_current_update_events().collect::<Vec<_>>();
        // when the inventory is read-only we do not emit a drop event
        assert_eq!(events.len(), 0);

        // Also make sure that the player inventory was not updated (as it is
        // read-only).
        let expected_player_slot_id = convert_to_player_slot_id(InventoryKind::Generic9x3, 50);
        assert_eq!(
            player_inventory.slot(expected_player_slot_id),
            // Inventory is read-only, item is not being dropped
            &ItemStack::new(ItemKind::IronIngot, 32)
        );
    }
}

/// The item stack should be dropped successfully, if the player has an
/// inventory open and the slot id points to his inventory.
#[test]
fn should_drop_item_stack_player_open_inventory_with_dropkey() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    app.update();

    let mut inventory = app
        .world_mut()
        .get_mut::<Inventory>(client)
        .expect("could not find inventory");

    inventory.set_slot(
        convert_to_player_slot_id(InventoryKind::Generic9x3, 50),
        ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
            ItemComponent::CustomName("Custom Iron".into_text_component()),
            ItemComponent::Lore(vec![
                "Other Lore Line 1.".into_text_component(),
                "Other Lore Line 2.".into_text_component(),
            ]),
        ]),
    );

    let _inventory_ent = set_up_open_inventory(&mut app, client);

    app.update();
    helper.clear_received();

    let inv_state = app
        .world_mut()
        .get_mut::<ClientInventoryState>(client)
        .expect("could not find client");

    let state_id = inv_state.state_id().0;
    let window_id = inv_state.window_id();

    helper.send(&ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: 50, // pressing control, the whole stack is dropped
        button: 1,
        mode: ClickMode::DropKey,
        slot_changes: vec![SlotChange {
            idx: 50,
            stack: ItemStack::EMPTY,
        }
        .into()]
        .into(),
        carried_item: ItemStack::EMPTY.into(),
    });

    app.update();

    // Make assertions
    let events = app
        .world()
        .get_resource::<Events<DropItemStackEvent>>()
        .expect("expected drop item stack events");

    let player_inventory = app
        .world()
        .get::<Inventory>(client)
        .expect("could not find inventory");

    let events = events.iter_current_update_events().collect::<Vec<_>>();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].client, client);
    assert_eq!(
        events[0].from_slot,
        Some(convert_to_player_slot_id(InventoryKind::Generic9x3, 50))
    );
    assert_eq!(
        events[0].stack,
        ItemStack::new(ItemKind::IronIngot, 32).with_components(vec![
            ItemComponent::CustomName("Custom Iron".into_text_component()),
            ItemComponent::Lore(vec![
                "Other Lore Line 1.".into_text_component(),
                "Other Lore Line 2.".into_text_component(),
            ]),
        ])
    );

    // Also make sure that the player inventory was updated correctly.
    let expected_player_slot_id = convert_to_player_slot_id(InventoryKind::Generic9x3, 50);
    assert_eq!(
        player_inventory.slot(expected_player_slot_id),
        &ItemStack::EMPTY
    );
}

#[test]
fn dragging_items() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    app.update();
    helper.clear_received();

    let draggable_stack = ItemStack::new(ItemKind::Diamond, 64).with_components(vec![
        ItemComponent::CustomName("Draggable Diamond".into_text_component()),
        ItemComponent::Lore(vec![
            "Lore Line 1.".into_text_component(),
            "Lore Line 2.".into_text_component(),
        ]),
    ]);

    let carried_stack = ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
        ItemComponent::CustomName("Draggable Diamond".into_text_component()),
        ItemComponent::Lore(vec![
            "Lore Line 1.".into_text_component(),
            "Lore Line 2.".into_text_component(),
        ]),
    ]);

    app.world_mut().get_mut::<CursorItem>(client).unwrap().0 = draggable_stack;

    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let window_id = inv_state.window_id();
    let state_id = inv_state.state_id().0;

    let drag_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
        button: 2,
        mode: ClickMode::Drag,
        slot_changes: vec![
            SlotChange {
                idx: 9,
                stack: ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
                    ItemComponent::CustomName("Draggable Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 10,
                stack: ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
                    ItemComponent::CustomName("Draggable Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 11,
                stack: ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
                    ItemComponent::CustomName("Draggable Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
        ]
        .into(),
        carried_item: carried_stack.clone().into(),
    };
    helper.send(&drag_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(
        sent_packets.0.len(),
        0,
        "Server should not send any packets for valid drag, got: {}",
        describe_packet_ids(&sent_packets)
    );

    let cursor_item = app
        .world_mut()
        .get::<CursorItem>(client)
        .expect("could not find client");

    assert_eq!(cursor_item.0, carried_stack);

    let inventory = app
        .world_mut()
        .get::<Inventory>(client)
        .expect("could not find inventory");

    let expected_slot_stack = ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
        ItemComponent::CustomName("Draggable Diamond".into_text_component()),
        ItemComponent::Lore(vec![
            "Lore Line 1.".into_text_component(),
            "Lore Line 2.".into_text_component(),
        ]),
    ]);

    for i in 9..12 {
        assert_eq!(inventory.slot(i), &expected_slot_stack);
    }
}

// If you drag a item stack across multiple slots, the mc client will send
// packets for each slot that you drag over + one final packet is sent that
// contains the slot changes. this test verifies the entire process. (Note: the
// packets are sent in that order, but only sent on drag release for some
// reason. So without drag release, no packets are sent) see https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container
// for the correct "button numbers" for the different drag actions.

#[test]
fn dragging_items_left_click_no_remainder() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();
    app.update();
    helper.clear_received();

    // setup
    // Slot 9: 64 Diamonds with NBT
    // pick up slot 9
    // (start drag) + Drag over slots 10,11,12,13 (one by one in that order)
    // end drag

    let item_stack = ItemStack::new(ItemKind::Diamond, 64).with_components(vec![
        ItemComponent::CustomName("Custom Diamond".into_text_component()),
        ItemComponent::Lore(vec![
            "Lore Line 1.".into_text_component(),
            "Lore Line 2.".into_text_component(),
        ]),
    ]);

    app.world_mut()
        .get_mut::<Inventory>(client)
        .unwrap()
        .set_slot(9, item_stack.clone());

    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let window_id = inv_state.window_id();
    let state_id = inv_state.state_id().0;

    // pickup diamonds
    let pick_up_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: 9,
        button: 0,
        mode: ClickMode::Click,
        slot_changes: vec![SlotChange {
            idx: 9,
            stack: ItemStack::EMPTY,
        }
        .into()]
        .into(),
        carried_item: item_stack.clone().into(),
    };
    helper.send(&pick_up_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(
        sent_packets.0.len(),
        0,
        "Server should not send any packets for valid click, got: {}",
        describe_packet_ids(&sent_packets)
    );

    // Ensure diamnods are in the cursor
    let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
    assert_eq!(cursor_item.0, item_stack, "Cursor should have the items");

    let start_drag_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
        button: 0, // start left click drag
        mode: ClickMode::Drag,
        slot_changes: vec![].into(),
        carried_item: item_stack.clone().into(),
    };
    helper.send(&start_drag_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(
        sent_packets.0.len(),
        0,
        "Server should not send any packets for valid drag start"
    );

    // cursor should still have the items
    let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
    assert_eq!(
        cursor_item.0, item_stack,
        "Cursor should still have items after drag start"
    );

    // sequentially "drag over the slots one by one"
    for slot in [10, 11, 12, 13] {
        let add_slot_packet = ContainerClickC2s {
            window_id,
            state_id: VarInt(state_id),
            slot_idx: slot,
            button: 1, // Add slot to left-click drag
            mode: ClickMode::Drag,
            slot_changes: vec![].into(),
            carried_item: item_stack.clone().into(),
        };
        helper.send(&add_slot_packet);

        app.update();
        let sent_packets = helper.collect_received();
        assert_eq!(
            sent_packets.0.len(),
            0,
            "Server should not send any packets for valid drag add slot {slot}"
        );

        // cursor should still have the items
        let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
        assert_eq!(
            cursor_item.0, item_stack,
            "Cursor should still have items after adding slot {slot} to drag"
        );
    }

    // End drag (release left mouse), results in a final packet with all slot
    // changes
    let end_drag_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
        button: 2, // end left click drag
        mode: ClickMode::Drag,
        slot_changes: vec![
            SlotChange {
                idx: 10,
                stack: ItemStack::new(ItemKind::Diamond, 16).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 11,
                stack: ItemStack::new(ItemKind::Diamond, 16).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 12,
                stack: ItemStack::new(ItemKind::Diamond, 16).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 13,
                stack: ItemStack::new(ItemKind::Diamond, 16).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
        ]
        .into(),
        carried_item: ItemStack::EMPTY.into(), /* cursor is empty after drag end (because we have
                                                * no remainder when dragging across 4 slots) */
    };
    helper.send(&end_drag_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(sent_packets.0.len(), 0, "Server should not resync");

    let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
    assert_eq!(cursor_item.0, ItemStack::EMPTY, "Cursor should be empty");

    let inventory = app.world_mut().get::<Inventory>(client).unwrap();

    // slot 9 (where the original stack was) should now be empty
    assert_eq!(inventory.slot(9), &ItemStack::EMPTY);

    // slots 10-13 should each have 16 diamonds with correct nbt
    for slot in [10, 11, 12, 13] {
        let expected = ItemStack::new(ItemKind::Diamond, 16).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore Line 2.".into_text_component(),
            ]),
        ]);
        assert_eq!(
            inventory.slot(slot),
            &expected,
            "Slot {slot} should have 16 diamonds"
        );
    }
}

#[test]
fn dragging_items_left_click_with_remainder() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();
    app.update();
    helper.clear_received();

    // setup
    // Slot 9: 64 Diamonds with NBT
    // Drag over slots 10,11,12 (3 slots)
    // 64 / 3 = 21 per slot, remainder 1
    // after end drag cursor should have 1 diamond remainng

    let item_stack = ItemStack::new(ItemKind::Diamond, 64).with_components(vec![
        ItemComponent::CustomName("Custom Diamond".into_text_component()),
        ItemComponent::Lore(vec![
            "Lore Line 1.".into_text_component(),
            "Lore Line 2.".into_text_component(),
        ]),
    ]);

    app.world_mut()
        .get_mut::<Inventory>(client)
        .unwrap()
        .set_slot(9, item_stack.clone());

    let inv_state = app.world_mut().get::<ClientInventoryState>(client).unwrap();
    let window_id = inv_state.window_id();
    let state_id = inv_state.state_id().0;

    // pickup diamonds
    let pick_up_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: 9,
        button: 0,
        mode: ClickMode::Click,
        slot_changes: vec![SlotChange {
            idx: 9,
            stack: ItemStack::EMPTY,
        }
        .into()]
        .into(),
        carried_item: item_stack.clone().into(),
    };
    helper.send(&pick_up_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(
        sent_packets.0.len(),
        0,
        "Server should not send any packets for valid click, got: {}",
        describe_packet_ids(&sent_packets)
    );

    // Ensure diamonds are in the cursor
    let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
    assert_eq!(cursor_item.0, item_stack, "Cursor should have the items");

    let start_drag_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
        button: 0, // start left click drag
        mode: ClickMode::Drag,
        slot_changes: vec![].into(),
        carried_item: item_stack.clone().into(),
    };
    helper.send(&start_drag_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(
        sent_packets.0.len(),
        0,
        "Server should not send any packets for valid drag start"
    );

    // cursor should still have the items
    let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
    assert_eq!(
        cursor_item.0, item_stack,
        "Cursor should still have items after drag start"
    );

    // sequentially "drag over the slots one by one" (only 3 slots this time)
    for slot in [10, 11, 12] {
        let add_slot_packet = ContainerClickC2s {
            window_id,
            state_id: VarInt(state_id),
            slot_idx: slot,
            button: 1, // Add slot to left-click drag
            mode: ClickMode::Drag,
            slot_changes: vec![].into(),
            carried_item: item_stack.clone().into(),
        };
        helper.send(&add_slot_packet);

        app.update();
        let sent_packets = helper.collect_received();
        assert_eq!(
            sent_packets.0.len(),
            0,
            "Server should not send any packets for valid drag add slot {slot}"
        );

        // cursor should still have the items
        let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
        assert_eq!(
            cursor_item.0, item_stack,
            "Cursor should still have items after adding slot {slot} to drag"
        );
    }

    // End drag
    // 64 items across 3 slots = 21 per slot, 1 remaining in cursor
    let end_drag_packet = ContainerClickC2s {
        window_id,
        state_id: VarInt(state_id),
        slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
        button: 2, // end left click drag
        mode: ClickMode::Drag,
        slot_changes: vec![
            SlotChange {
                idx: 10,
                stack: ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 11,
                stack: ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
            SlotChange {
                idx: 12,
                stack: ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
                    ItemComponent::CustomName("Custom Diamond".into_text_component()),
                    ItemComponent::Lore(vec![
                        "Lore Line 1.".into_text_component(),
                        "Lore Line 2.".into_text_component(),
                    ]),
                ]),
            }
            .into(),
        ]
        .into(),
        // cursor has 1 diamond remaining
        carried_item: ItemStack::new(ItemKind::Diamond, 1)
            .with_components(vec![
                ItemComponent::CustomName("Custom Diamond".into_text_component()),
                ItemComponent::Lore(vec![
                    "Lore Line 1.".into_text_component(),
                    "Lore Line 2.".into_text_component(),
                ]),
            ])
            .into(),
    };
    helper.send(&end_drag_packet);

    app.update();
    let sent_packets = helper.collect_received();
    assert_eq!(sent_packets.0.len(), 0, "Server should not resync");

    // cursor should have 1 diamond remaining
    let cursor_item = app.world_mut().get::<CursorItem>(client).unwrap();
    let expected_cursor = ItemStack::new(ItemKind::Diamond, 1).with_components(vec![
        ItemComponent::CustomName("Custom Diamond".into_text_component()),
        ItemComponent::Lore(vec![
            "Lore Line 1.".into_text_component(),
            "Lore Line 2.".into_text_component(),
        ]),
    ]);
    assert_eq!(
        cursor_item.0, expected_cursor,
        "Cursor should have 1 diamond remaining"
    );

    let inventory = app.world_mut().get::<Inventory>(client).unwrap();

    // slot 9 (where the original stack was) should now be empty
    assert_eq!(inventory.slot(9), &ItemStack::EMPTY);

    // slots 10-12 should each have 21 diamonds with correct nbt
    for slot in [10, 11, 12] {
        let expected = ItemStack::new(ItemKind::Diamond, 21).with_components(vec![
            ItemComponent::CustomName("Custom Diamond".into_text_component()),
            ItemComponent::Lore(vec![
                "Lore Line 1.".into_text_component(),
                "Lore Line 2.".into_text_component(),
            ]),
        ]);
        assert_eq!(
            inventory.slot(slot),
            &expected,
            "Slot {slot} should have 21 diamonds"
        );
    }
}
