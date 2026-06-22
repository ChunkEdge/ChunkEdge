use chunkedge_inventory::{HeldItem, Inventory, PlayerAction};
use chunkedge_server::entity::living::LivingFlags;
use chunkedge_server::event_loop::PacketEvent;
use chunkedge_server::interact_item::InteractItemEvent;
use chunkedge_server::protocol::packets::play::PlayerActionC2s;
use chunkedge_server::ItemKind;

use super::*;

/// This component will broadcast item interactions (e.g. drawing a bow, eating
/// food) to other players using `LivingFlags::set_using_item`.
#[derive(Debug, Default, Clone, Component)]
pub struct EquipmentInteractionBroadcast;

// Sets flag to true when the client starts interacting with an
// item.
pub(crate) fn start_interaction(
    mut clients: Query<
        (&Inventory, &HeldItem, &mut LivingFlags),
        (With<Client>, With<EquipmentInteractionBroadcast>),
    >,
    mut events: MessageReader<InteractItemEvent>,
) {
    for event in events.read() {
        if let Ok((inv, held_item, mut flags)) = clients.get_mut(event.client) {
            let item = inv.slot(held_item.slot()).item;
            let has_arrows = inv.first_slot_with_item(ItemKind::Arrow, i8::MAX).is_some();
            if (item == ItemKind::Bow && !has_arrows)
                || (item == ItemKind::Crossbow
                    && !has_arrows
                    && inv.slot(45).item != ItemKind::FireworkRocket)
            {
                continue;
            }
            flags.set_using_item(true);
        }
    }
}

// Sets flag to false when the client stops interacting with an
// item.
pub(crate) fn stop_interaction(
    mut clients: Query<&mut LivingFlags, (With<Client>, With<EquipmentInteractionBroadcast>)>,
    mut packets: MessageReader<PacketEvent>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<PlayerActionC2s>() {
            if pkt.action == PlayerAction::ReleaseUseItem {
                if let Ok(mut flags) = clients.get_mut(packet.client) {
                    flags.set_using_item(false);
                }
            }
        }
    }
}
