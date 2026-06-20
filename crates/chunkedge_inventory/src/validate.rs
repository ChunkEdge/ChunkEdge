use chunkedge_server::protocol::anyhow::{self, ensure};
use chunkedge_server::protocol::packets::play::container_click_c2s::{ClickMode, SlotChange};
use chunkedge_server::protocol::packets::play::ContainerClickC2s;
use chunkedge_server::protocol::VarInt;
use chunkedge_server::ItemStack;

use crate::player_inventory::PlayerInventory;
use crate::validate::anyhow::bail;
use crate::{CursorItem, Inventory, InventoryWindow};

/// This function simulates the "item click" action on the server
/// and validates it.
/// If the action is valid: `Ok`,
/// We return the updated cursor item and the slot changes.
///
/// We need to compute those values in the validation because the packet no
/// longer contains this data (item stacks are hashed now).
pub(super) fn validate_click_slot_packet(
    packet: &ContainerClickC2s,
    player_inventory: &Inventory,
    open_inventory: Option<&Inventory>,
    cursor_item: &CursorItem,
) -> anyhow::Result<(ItemStack, Vec<SlotChange>)> {
    ensure!(
        (packet.window_id == VarInt(0)) == open_inventory.is_none(),
        "window id and open inventory mismatch: window_id: {} open_inventory: {}",
        packet.window_id.0,
        open_inventory.is_some()
    );

    let mut new_slot_changes = Vec::with_capacity(packet.slot_changes.len());

    let max_slot = if let Some(open_inv) = open_inventory {
        // when the window is split, we can only access the main slots of player's
        // inventory
        PlayerInventory::MAIN_SIZE + open_inv.slot_count()
    } else {
        player_inventory.slot_count()
    };

    // check all slot ids and item counts are valid
    ensure!(
        packet.slot_changes.iter().all(|s| {
            if !(0..=max_slot).contains(&(s.idx as u16)) {
                return false;
            }

            if !s.stack.is_empty() {
                let max_stack_size = s.stack.item.max_stack().max(s.stack.count);
                if !(1..=max_stack_size).contains(&(s.stack.count)) {
                    return false;
                }
            }

            true
        }),
        "invalid slot ids or item counts"
    );

    // check carried item count is valid
    if !packet.carried_item.is_empty() {
        let carried_item = &packet.carried_item;

        let max_stack_size = carried_item.item.max_stack().max(carried_item.count);
        ensure!(
            (1..=max_stack_size).contains(&carried_item.count),
            "invalid carried item count"
        );
    }

    match packet.mode {
        ClickMode::Click => {
            ensure!((0..=1).contains(&packet.button), "invalid button");
            ensure!(
                (0..=max_slot).contains(&(packet.slot_idx as u16))
                    || packet.slot_idx == PlayerInventory::SLOT_OUTSIDE_INVENTORY
                    || packet.slot_idx == -1,
                "invalid slot index"
            )
        }
        ClickMode::ShiftClick => {
            ensure!((0..=1).contains(&packet.button), "invalid button");
            ensure!(
                packet.carried_item.is_empty(),
                "carried item must be empty for a hotbar swap"
            );
            ensure!(
                (0..=max_slot).contains(&(packet.slot_idx as u16)),
                "invalid slot index"
            )
        }
        ClickMode::Hotbar => {
            ensure!(matches!(packet.button, 0..=8 | 40), "invalid button");
            ensure!(
                packet.carried_item.is_empty(),
                "carried item must be empty for a hotbar swap"
            );
        }
        ClickMode::CreativeMiddleClick => {
            ensure!(packet.button == 2, "invalid button");
            ensure!(
                (0..=max_slot).contains(&(packet.slot_idx as u16)),
                "invalid slot index"
            )
        }
        ClickMode::DropKey => {
            ensure!((0..=1).contains(&packet.button), "invalid button");
            ensure!(
                packet.carried_item.is_empty(),
                "carried item must be empty for an item drop"
            );
            ensure!(
                (0..=max_slot).contains(&(packet.slot_idx as u16))
                    || packet.slot_idx == PlayerInventory::SLOT_OUTSIDE_INVENTORY,
                "invalid slot index"
            )
        }
        ClickMode::Drag => {
            ensure!(
                matches!(packet.button, 0..=2 | 4..=6 | 8..=10),
                "invalid button"
            );
            ensure!(
                (0..=max_slot).contains(&(packet.slot_idx as u16))
                    || packet.slot_idx == PlayerInventory::SLOT_OUTSIDE_INVENTORY,
                "invalid slot index"
            )
        }
        ClickMode::DoubleClick => ensure!(packet.button == 0, "invalid button"),
    }

    // Check that items aren't being duplicated, i.e. conservation of mass.

    let window = InventoryWindow {
        player_inventory,
        open_inventory,
    };

    let mut new_cursor_stack = ItemStack::EMPTY;

    match packet.mode {
        ClickMode::Click => {
            if packet.slot_idx == -1 {
                // Clicked outside the allowed window
                ensure!(
                    packet.slot_changes.is_empty(),
                    "slot modifications must be empty"
                );

                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );

                // Margin clicks do not modify the carried stack.
                new_cursor_stack = cursor_item.0.clone();
            } else if packet.slot_idx == PlayerInventory::SLOT_OUTSIDE_INVENTORY {
                // Clicked outside the window, so the client is dropping an item
                ensure!(
                    packet.slot_changes.is_empty(),
                    "slot modifications must be empty"
                );

                // Clicked outside the window
                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                let expected_delta = match packet.button {
                    1 => -1,
                    0 => {
                        if !cursor_item.is_empty() {
                            -i32::from(cursor_item.0.count)
                        } else {
                            0
                        }
                    }
                    _ => unreachable!(),
                };
                ensure!(
                    count_deltas == expected_delta,
                    "invalid item delta: expected {expected_delta}, got {count_deltas}"
                );
            } else {
                // If the user clicked on an empty slot for example
                if packet.slot_changes.is_empty() {
                    let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                    ensure!(
                        count_deltas == 0,
                        "invalid item delta: expected 0, got {count_deltas}"
                    );

                    ensure!(
                        packet.carried_item.item == cursor_item.0.item
                            && packet.carried_item.count == cursor_item.0.count,
                        "carried item must remain unchanged for a non-modifying click"
                    );

                    new_cursor_stack = cursor_item.0.clone();
                } else {
                    ensure!(
                        packet.slot_changes.len() == 1,
                        "click must modify one slot, got {}",
                        packet.slot_changes.len()
                    );

                    let hashed_change = &packet.slot_changes[0];
                    let old_slot = window.slot(hashed_change.idx as u16);
                    // TODO: make sure NBT is the same.
                    //       Sometimes, the client will add nbt data to an item if it's missing,
                    // like       "Damage" to a sword.
                    let should_swap: bool = packet.button == 0
                        && match (!old_slot.is_empty(), !cursor_item.is_empty()) {
                            (true, true) => old_slot.item != cursor_item.item,
                            (true, false) => true,
                            (false, true) => cursor_item.count <= cursor_item.item.max_stack(),
                            (false, false) => false,
                        };

                    if should_swap {
                        // assert that a swap occurs
                        ensure!(
                            // There are some cases where the client will add NBT data that
                            // did not previously exist.
                            old_slot.item == packet.carried_item.item
                                && old_slot.count == packet.carried_item.count
                                && cursor_item.0.item == hashed_change.stack.item
                                && cursor_item.0.count == hashed_change.stack.count,
                            "swapped items must match"
                        );

                        // Find the unhashed cursor
                        new_slot_changes.push(SlotChange {
                            idx: hashed_change.idx,
                            stack: cursor_item.0.clone().with_count(hashed_change.stack.count),
                        });
                        new_cursor_stack = old_slot.clone().with_count(packet.carried_item.count);
                    } else {
                        // assert that a merge occurs
                        let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                        ensure!(
                            count_deltas == 0,
                            "invalid item delta for stack merge: {count_deltas}"
                        );

                        // Find unhashed clicked slot
                        let unhashed_slot = if !hashed_change.stack.is_empty() {
                            if cursor_item.item == hashed_change.stack.item
                                && !cursor_item.is_empty()
                            {
                                cursor_item.0.clone().with_count(hashed_change.stack.count)
                            } else if old_slot.item == hashed_change.stack.item
                                && !old_slot.is_empty()
                            {
                                old_slot.clone().with_count(hashed_change.stack.count)
                            } else {
                                bail!("could not find unhashed click item");
                            }
                        } else {
                            ItemStack::EMPTY
                        };

                        new_slot_changes.push(SlotChange {
                            idx: hashed_change.idx,
                            stack: unhashed_slot,
                        });

                        // Find unhashed cursor
                        if !packet.carried_item.is_empty() {
                            new_cursor_stack = if cursor_item.item == packet.carried_item.item
                                && !cursor_item.is_empty()
                            {
                                cursor_item.0.clone().with_count(packet.carried_item.count)
                            } else if old_slot.item == packet.carried_item.item
                                && !old_slot.is_empty()
                            {
                                old_slot.clone().with_count(packet.carried_item.count)
                            } else {
                                bail!("could not unhash carried item");
                            };
                        }
                    }
                }
            }
        }
        ClickMode::ShiftClick => {
            // If the user clicked on an empty slot for example
            if packet.slot_changes.is_empty() {
                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );
            } else {
                ensure!(
                    (2..=3).contains(&packet.slot_changes.len()),
                    "shift click must modify 2 or 3 slots, got {}",
                    packet.slot_changes.len()
                );

                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );

                let Some(item_kind) = packet
                    .slot_changes
                    .iter()
                    .find(|s| !s.stack.is_empty())
                    .map(|s| s.stack.item)
                else {
                    bail!("shift click must move an item");
                };

                let source_slot = window.slot(packet.slot_idx as u16);
                ensure!(
                    source_slot.item == item_kind,
                    "shift click must move the same item kind as modified slots"
                );

                // assert all moved items are the same kind
                ensure!(
                    packet
                        .slot_changes
                        .iter()
                        .filter(|s| !s.stack.is_empty())
                        .all(|s| s.stack.item == item_kind),
                    "shift click must move the same item kind"
                );

                // Find unhashed slots
                for hashed_change in packet.slot_changes.iter() {
                    let unhashed = if !hashed_change.stack.is_empty() {
                        source_slot.clone().with_count(hashed_change.stack.count)
                    } else {
                        ItemStack::EMPTY
                    };

                    new_slot_changes.push(SlotChange {
                        idx: hashed_change.idx,
                        stack: unhashed,
                    });
                }
            }
        }

        ClickMode::Hotbar => {
            if packet.slot_changes.is_empty() {
                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );
            } else {
                ensure!(
                    packet.slot_changes.len() == 2,
                    "hotbar swap must modify two slots, got {}",
                    packet.slot_changes.len()
                );

                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );

                // assert that a swap occurs
                let old_slots = [
                    window.slot(packet.slot_changes[0].idx as u16),
                    window.slot(packet.slot_changes[1].idx as u16),
                ];
                // There are some cases where the client will add NBT data that did not
                // previously exist.
                ensure!(
                    old_slots
                        .iter()
                        .any(|s| s.item == packet.slot_changes[0].stack.item
                            && s.count == packet.slot_changes[0].stack.count)
                        && old_slots
                            .iter()
                            .any(|s| s.item == packet.slot_changes[1].stack.item
                                && s.count == packet.slot_changes[1].stack.count),
                    "swapped items must match"
                );

                // find unhashed swapped slots
                for (i, hashed_change) in packet.slot_changes.iter().enumerate() {
                    let other_slot = old_slots[1 - i];
                    let unhashed = if !hashed_change.stack.is_empty() {
                        if other_slot.item == hashed_change.stack.item && !other_slot.is_empty() {
                            other_slot.clone().with_count(hashed_change.stack.count)
                        } else {
                            bail!("could not find unhashed hotbar swap item");
                        }
                    } else {
                        ItemStack::EMPTY
                    };

                    new_slot_changes.push(SlotChange {
                        idx: hashed_change.idx,
                        stack: unhashed,
                    });
                }
            }
        }
        ClickMode::CreativeMiddleClick => {}
        ClickMode::DropKey => {
            if packet.slot_changes.is_empty() {
                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );
            } else {
                ensure!(
                    packet.slot_changes.len() == 1,
                    "drop key must modify exactly one slot"
                );
                ensure!(
                    packet.slot_idx == packet.slot_changes.first().map_or(-2, |s| s.idx),
                    "slot index does not match modified slot"
                );

                let hashed_change = &packet.slot_changes[0];
                let old_slot = window.slot(packet.slot_idx as u16);
                let new_slot = &hashed_change.stack;
                let is_transmuting = match (!old_slot.is_empty(), !new_slot.is_empty()) {
                    // TODO: make sure NBT is the same.
                    // Sometimes, the client will add nbt data to an item if it's missing, like
                    // "Damage" to a sword.
                    (true, true) => old_slot.item != new_slot.item,
                    (_, false) => false,
                    (false, true) => true,
                };
                ensure!(!is_transmuting, "transmuting items is not allowed");

                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);

                let expected_delta = match packet.button {
                    0 => -1,
                    1 => {
                        if !old_slot.is_empty() {
                            -i32::from(old_slot.count)
                        } else {
                            0
                        }
                    }
                    _ => unreachable!(),
                };
                ensure!(
                    count_deltas == expected_delta,
                    "invalid item delta: expected {expected_delta}, got {count_deltas}"
                );

                // FInd unhashed slot
                let unhashed = if !new_slot.is_empty() {
                    old_slot.clone().with_count(new_slot.count)
                } else {
                    ItemStack::EMPTY
                };

                new_slot_changes.push(SlotChange {
                    idx: hashed_change.idx,
                    stack: unhashed,
                });
            }
        }
        ClickMode::Drag => {
            if matches!(packet.button, 2 | 6 | 10) {
                // TODO: create constants or enum for the button ids
                // buttons 2, 6, 10 mean we are ending the drag
                let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
                ensure!(
                    count_deltas == 0,
                    "invalid item delta: expected 0, got {count_deltas}"
                );

                // Items are spread out from a stack, to get the real itemstack we need to find
                // the current (serverside) cursor.
                for hashed_change in packet.slot_changes.iter() {
                    let current_slot = window.slot(hashed_change.idx as u16);

                    let unhashed = if !hashed_change.stack.is_empty() {
                        if cursor_item.item == hashed_change.stack.item && !cursor_item.is_empty() {
                            cursor_item.0.clone().with_count(hashed_change.stack.count)
                        } else if current_slot.item == hashed_change.stack.item
                            && !current_slot.is_empty()
                        {
                            current_slot.clone().with_count(hashed_change.stack.count)
                        } else {
                            bail!("could not find unhashed drag item");
                        }
                    } else {
                        bail!("could not find unhashed drag item");
                    };

                    new_slot_changes.push(SlotChange {
                        idx: hashed_change.idx,
                        stack: unhashed,
                    });
                }

                // find unhashed cursor
                if !packet.carried_item.is_empty() {
                    new_cursor_stack = if cursor_item.item == packet.carried_item.item
                        && !cursor_item.is_empty()
                    {
                        cursor_item.0.clone().with_count(packet.carried_item.count)
                    } else {
                        bail!("could not unhash carried item for drag");
                    };
                }
            } else {
                // We are currently dragging or starting to drag (buttons: 0, 4, 8, 1, 5, 9)
                ensure!(
                    packet.slot_changes.is_empty()
                        && packet.carried_item.item == cursor_item.0.item
                        && packet.carried_item.count == cursor_item.0.count,
                    "invalid drag state"
                );

                // the cursor only changes if the drag is released
                new_cursor_stack = cursor_item.0.clone();
            }
        }
        ClickMode::DoubleClick => {
            let count_deltas = calculate_net_item_delta(packet, &window, cursor_item);
            ensure!(
                count_deltas == 0,
                "invalid item delta: expected 0, got {count_deltas}"
            );

            // Items are collected into a stack
            // TODO: testing
            for hashed_change in packet.slot_changes.iter() {
                let current_slot = window.slot(hashed_change.idx as u16);
                let unhashed = if !hashed_change.stack.is_empty() {
                    current_slot.clone().with_count(hashed_change.stack.count)
                } else {
                    ItemStack::EMPTY
                };

                new_slot_changes.push(SlotChange {
                    idx: hashed_change.idx,
                    stack: unhashed,
                });
            }

            // find unhsashd cursor
            if !packet.carried_item.is_empty() {
                new_cursor_stack =
                    if cursor_item.item == packet.carried_item.item && !cursor_item.is_empty() {
                        cursor_item.0.clone().with_count(packet.carried_item.count)
                    } else {
                        // Look for the item in the modified slots
                        let source_stack = packet.slot_changes.iter().find_map(|change| {
                            let slot = window.slot(change.idx as u16);
                            if slot.item == packet.carried_item.item && !slot.is_empty() {
                                Some(slot.clone())
                            } else {
                                None
                            }
                        });

                        if let Some(source) = source_stack {
                            source.with_count(packet.carried_item.count)
                        } else {
                            bail!("could not unhash carried item for double click");
                        }
                    };
            }
        }
    }

    Ok((new_cursor_stack, new_slot_changes))
}

/// Calculate the total difference in item counts if the changes in this packet
/// were to be applied.
///
/// Returns a positive number if items were added to the window, and a negative
/// number if items were removed from the window.
fn calculate_net_item_delta(
    packet: &ContainerClickC2s,
    window: &InventoryWindow,
    cursor_item: &CursorItem,
) -> i32 {
    let mut net_item_delta: i32 = 0;

    for slot in packet.slot_changes.iter() {
        let old_slot = window.slot(slot.idx as u16);
        let new_slot = &slot.stack;

        net_item_delta += match (!old_slot.is_empty(), !new_slot.is_empty()) {
            (true, true) => i32::from(new_slot.count) - i32::from(old_slot.count),
            (true, false) => -i32::from(old_slot.count),
            (false, true) => i32::from(new_slot.count),
            (false, false) => 0,
        };
    }

    net_item_delta += match (!cursor_item.is_empty(), !packet.carried_item.is_empty()) {
        (true, true) => i32::from(packet.carried_item.count) - i32::from(cursor_item.count),
        (true, false) => -i32::from(cursor_item.count),
        (false, true) => i32::from(packet.carried_item.count),
        (false, false) => 0,
    };

    net_item_delta
}
// TODO: validate nbt after validation
#[cfg(test)]
mod tests {
    use chunkedge_server::protocol::VarInt;
    use chunkedge_server::{ItemKind, ItemStack};

    use super::*;
    use crate::InventoryKind;

    #[test]
    fn net_item_delta_1() {
        let drag_packet = ContainerClickC2s {
            window_id: VarInt(2),
            state_id: VarInt(14),
            slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
            button: 2,
            mode: ClickMode::Drag,
            slot_changes: vec![
                SlotChange {
                    idx: 4,
                    stack: ItemStack::new(ItemKind::Diamond, 21),
                }
                .into(),
                SlotChange {
                    idx: 3,
                    stack: ItemStack::new(ItemKind::Diamond, 21),
                }
                .into(),
                SlotChange {
                    idx: 5,
                    stack: ItemStack::new(ItemKind::Diamond, 21),
                }
                .into(),
            ]
            .into(),
            carried_item: ItemStack::new(ItemKind::Diamond, 1).into(),
        };

        let player_inventory = Inventory::new(InventoryKind::Player);
        let inventory = Inventory::new(InventoryKind::Generic9x1);
        let window = InventoryWindow::new(&player_inventory, Some(&inventory));
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Diamond, 64));

        assert_eq!(
            calculate_net_item_delta(&drag_packet, &window, &cursor_item),
            0
        );
    }

    #[test]
    fn net_item_delta_2() {
        let drag_packet = ContainerClickC2s {
            window_id: VarInt(2),
            state_id: VarInt(14),
            slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
            button: 2,
            mode: ClickMode::Click,
            slot_changes: vec![
                SlotChange {
                    idx: 2,
                    stack: ItemStack::new(ItemKind::Diamond, 2),
                }
                .into(),
                SlotChange {
                    idx: 3,
                    stack: ItemStack::new(ItemKind::IronIngot, 2),
                }
                .into(),
                SlotChange {
                    idx: 4,
                    stack: ItemStack::new(ItemKind::GoldIngot, 2),
                }
                .into(),
                SlotChange {
                    idx: 5,
                    stack: ItemStack::new(ItemKind::Emerald, 2),
                }
                .into(),
            ]
            .into(),
            carried_item: ItemStack::new(ItemKind::OakWood, 2).into(),
        };

        let player_inventory = Inventory::new(InventoryKind::Player);
        let inventory = Inventory::new(InventoryKind::Generic9x1);
        let window = InventoryWindow::new(&player_inventory, Some(&inventory));
        let cursor_item = CursorItem::default();

        assert_eq!(
            calculate_net_item_delta(&drag_packet, &window, &cursor_item),
            10
        );
    }

    #[test]
    fn click_filled_slot_with_empty_cursor_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let mut inventory = Inventory::new(InventoryKind::Generic9x1);
        inventory.set_slot(0, ItemStack::new(ItemKind::Diamond, 20));
        let cursor_item = CursorItem::default();
        let packet = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::EMPTY,
            }
            .into()]
            .into(),
            carried_item: inventory.slot(0).clone().into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, Some(&inventory), &cursor_item)
            .expect("packet should be valid");
    }

    #[test]
    fn click_slot_with_filled_cursor_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let inventory1 = Inventory::new(InventoryKind::Generic9x1);
        let mut inventory2 = Inventory::new(InventoryKind::Generic9x1);
        inventory2.set_slot(0, ItemStack::new(ItemKind::Diamond, 10));
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Diamond, 20));
        let packet1 = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 20),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        };
        let packet2 = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 30),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        };

        validate_click_slot_packet(&packet1, &player_inventory, Some(&inventory1), &cursor_item)
            .expect("packet should be valid");

        validate_click_slot_packet(&packet2, &player_inventory, Some(&inventory2), &cursor_item)
            .expect("packet should be valid");
    }

    #[test]
    fn click_filled_slot_with_filled_cursor_stack_overflow_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let mut inventory = Inventory::new(InventoryKind::Generic9x1);
        inventory.set_slot(0, ItemStack::new(ItemKind::Diamond, 20));
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Diamond, 64));
        let packet = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 64),
            }
            .into()]
            .into(),
            carried_item: ItemStack::new(ItemKind::Diamond, 20).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, Some(&inventory), &cursor_item)
            .expect("packet should be valid");
    }

    #[test]
    fn click_filled_slot_with_filled_cursor_different_item_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let mut inventory = Inventory::new(InventoryKind::Generic9x1);
        inventory.set_slot(0, ItemStack::new(ItemKind::IronIngot, 2));
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Diamond, 2));
        let packet = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 2),
            }
            .into()]
            .into(),
            carried_item: ItemStack::new(ItemKind::IronIngot, 2).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, Some(&inventory), &cursor_item)
            .expect("packet should be valid");
    }

    #[test]
    fn click_slot_with_filled_cursor_failure() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let inventory1 = Inventory::new(InventoryKind::Generic9x1);
        let mut inventory2 = Inventory::new(InventoryKind::Generic9x1);
        inventory2.set_slot(0, ItemStack::new(ItemKind::Diamond, 10));
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Diamond, 20));
        let packet1 = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 22),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        };
        let packet2 = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![SlotChange {
                idx: 0,
                stack: ItemStack::new(ItemKind::Diamond, 32),
            }
            .into()]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        };
        let packet3 = ContainerClickC2s {
            window_id: VarInt(1),
            button: 0,
            mode: ClickMode::Click,
            state_id: VarInt(0),
            slot_idx: 0,
            slot_changes: vec![
                SlotChange {
                    idx: 0,
                    stack: ItemStack::new(ItemKind::Diamond, 22),
                }
                .into(),
                SlotChange {
                    idx: 1,
                    stack: ItemStack::new(ItemKind::Diamond, 22),
                }
                .into(),
            ]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        };

        validate_click_slot_packet(&packet1, &player_inventory, Some(&inventory1), &cursor_item)
            .expect_err("packet 1 should fail item duplication check");

        validate_click_slot_packet(&packet2, &player_inventory, Some(&inventory2), &cursor_item)
            .expect_err("packet 2 should fail item duplication check");

        validate_click_slot_packet(&packet3, &player_inventory, Some(&inventory1), &cursor_item)
            .expect_err("packet 3 should fail item duplication check");
    }

    #[test]
    fn disallow_item_transmutation() {
        // no alchemy allowed - make sure that lead can't be turned into gold

        let mut player_inventory = Inventory::new(InventoryKind::Player);
        player_inventory.set_slot(9, ItemStack::new(ItemKind::Lead, 2));
        let cursor_item = CursorItem::default();

        let packets = [
            ContainerClickC2s {
                window_id: VarInt(0),
                button: 0,
                mode: ClickMode::ShiftClick,
                state_id: VarInt(0),
                slot_idx: 9,
                slot_changes: vec![
                    SlotChange {
                        idx: 9,
                        stack: ItemStack::EMPTY,
                    }
                    .into(),
                    SlotChange {
                        idx: 36,
                        stack: ItemStack::new(ItemKind::GoldIngot, 2),
                    }
                    .into(),
                ]
                .into(),
                carried_item: ItemStack::EMPTY.into(),
            },
            ContainerClickC2s {
                window_id: VarInt(0),
                button: 0,
                mode: ClickMode::Hotbar,
                state_id: VarInt(0),
                slot_idx: 9,
                slot_changes: vec![
                    SlotChange {
                        idx: 9,
                        stack: ItemStack::EMPTY,
                    }
                    .into(),
                    SlotChange {
                        idx: 36,
                        stack: ItemStack::new(ItemKind::GoldIngot, 2),
                    }
                    .into(),
                ]
                .into(),
                carried_item: ItemStack::EMPTY.into(),
            },
            ContainerClickC2s {
                window_id: VarInt(0),
                button: 0,
                mode: ClickMode::Click,
                state_id: VarInt(0),
                slot_idx: 9,
                slot_changes: vec![SlotChange {
                    idx: 9,
                    stack: ItemStack::EMPTY,
                }
                .into()]
                .into(),
                carried_item: ItemStack::new(ItemKind::GoldIngot, 2).into(),
            },
            ContainerClickC2s {
                window_id: VarInt(0),
                button: 0,
                mode: ClickMode::DropKey,
                state_id: VarInt(0),
                slot_idx: 9,
                slot_changes: vec![SlotChange {
                    idx: 9,
                    stack: ItemStack::new(ItemKind::GoldIngot, 1),
                }
                .into()]
                .into(),
                carried_item: ItemStack::EMPTY.into(),
            },
        ];

        for (i, packet) in packets.iter().enumerate() {
            validate_click_slot_packet(packet, &player_inventory, None, &cursor_item).expect_err(
                &format!("packet {i} passed item duplication check when it should have failed"),
            );
        }
    }

    #[test]
    fn allow_shift_click_overflow_to_new_stack() {
        let mut player_inventory = Inventory::new(InventoryKind::Player);
        player_inventory.set_slot(9, ItemStack::new(ItemKind::Diamond, 64));
        player_inventory.set_slot(36, ItemStack::new(ItemKind::Diamond, 32));
        let cursor_item = CursorItem::default();

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: 9,
            button: 0,
            mode: ClickMode::ShiftClick,
            slot_changes: vec![
                SlotChange {
                    idx: 37,
                    stack: ItemStack::new(ItemKind::Diamond, 32),
                }
                .into(),
                SlotChange {
                    idx: 36,
                    stack: ItemStack::new(ItemKind::Diamond, 64),
                }
                .into(),
                SlotChange {
                    idx: 9,
                    stack: ItemStack::EMPTY,
                }
                .into(),
            ]
            .into(),
            carried_item: ItemStack::EMPTY.into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }

    #[test]
    fn allow_pickup_overfull_stack_click() {
        let mut player_inventory = Inventory::new(InventoryKind::Player);
        player_inventory.set_slot(9, ItemStack::new(ItemKind::Apple, 100));
        let cursor_item = CursorItem::default();

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: 9,
            button: 0,
            mode: ClickMode::Click,
            slot_changes: vec![SlotChange {
                idx: 9,
                stack: ItemStack::EMPTY,
            }
            .into()]
            .into(),
            carried_item: ItemStack::new(ItemKind::Apple, 100).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }

    #[test]
    fn allow_place_overfull_stack_click() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Apple, 100));

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: 9,
            button: 0,
            mode: ClickMode::Click,
            slot_changes: vec![SlotChange {
                idx: 9,
                stack: ItemStack::new(ItemKind::Apple, 64),
            }
            .into()]
            .into(),
            carried_item: ItemStack::new(ItemKind::Apple, 36).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }
    #[test]
    fn allow_clicking_outside_inventory_when_not_holding_anything_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Air, 0));

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
            button: 0,
            mode: ClickMode::DropKey, // when not holding an item and clicking outside the user
            // interface the client sends this kind of packet
            slot_changes: vec![].into(),
            carried_item: ItemStack::new(ItemKind::Air, 0).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }
    #[test]
    fn allow_clicking_outside_inventory_when_holding_something_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Air, 0));

        // This is in the notchian server a stack drop
        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: PlayerInventory::SLOT_OUTSIDE_INVENTORY,
            button: 0,
            mode: ClickMode::Click, // when holding an item its a click
            slot_changes: vec![].into(),
            carried_item: ItemStack::new(ItemKind::Air, 0).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }
    #[test]
    fn allow_clicking_on_the_margin_area_in_inventory_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Air, 0));

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: -1, // -1 here means on the margin areas
            button: 0,
            mode: ClickMode::Click,
            slot_changes: vec![].into(),
            carried_item: ItemStack::new(ItemKind::Air, 0).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }
    #[test]
    fn allow_clicking_on_an_empty_slot_with_empty_carried_item_success() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Air, 0));

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: 3,
            button: 0,
            mode: ClickMode::Click,
            slot_changes: vec![].into(),
            carried_item: ItemStack::new(ItemKind::Air, 0).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }
    #[test]
    fn allow_clicking_hotbar_keybinds_when_both_source_and_target_are_empty() {
        let player_inventory = Inventory::new(InventoryKind::Player);
        let cursor_item = CursorItem(ItemStack::new(ItemKind::Air, 0));

        let packet = ContainerClickC2s {
            window_id: VarInt(0),
            state_id: VarInt(2),
            slot_idx: 0,
            button: 0,
            mode: ClickMode::Hotbar,
            slot_changes: vec![].into(),
            carried_item: ItemStack::new(ItemKind::Air, 0).into(),
        };

        validate_click_slot_packet(&packet, &player_inventory, None, &cursor_item)
            .expect("packet should be valid");
    }
}
