use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_ecs::system::SystemState;
use chunkedge_entity::active_status_effects::{ActiveStatusEffect, ActiveStatusEffects};
use chunkedge_entity::entity::Flags;
use chunkedge_entity::living::PotionSwirlsAmbient;
use chunkedge_protocol::packets::play::{
    update_mob_effect_s2c, RemoveMobEffectS2c, UpdateMobEffectS2c,
};
use chunkedge_protocol::status_effects::StatusEffect;
use chunkedge_protocol::{VarInt, WritePacket};

use crate::client::Client;
use crate::EventLoopPostUpdate;

/// Event for when a status effect is added to an entity or the amplifier or
/// duration of an existing status effect is changed.
#[derive(Message, Clone, PartialEq, Eq, Debug)]
pub struct StatusEffectAdded {
    pub entity: Entity,
    pub status_effect: StatusEffect,
}

/// Event for when a status effect is removed from an entity.
#[derive(Message, Clone, PartialEq, Eq, Debug)]
pub struct StatusEffectRemoved {
    pub entity: Entity,
    pub status_effect: ActiveStatusEffect,
}

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StatusEffectAdded>()
            .add_message::<StatusEffectRemoved>()
            .add_systems(
                EventLoopPostUpdate,
                (
                    add_status_effects,
                    update_active_status_effects,
                    add_status_effects,
                ),
            );
    }
}

fn update_active_status_effects(
    world: &mut World,
    state: &mut SystemState<Query<&mut ActiveStatusEffects>>,
) {
    let mut query = state.get_mut(world);
    for mut active_status_effects in &mut query {
        active_status_effects.increment_active_ticks();
    }
}

fn create_packet(effect: &ActiveStatusEffect) -> UpdateMobEffectS2c {
    UpdateMobEffectS2c {
        entity_id: VarInt(0), // We reserve ID 0 for clients.
        effect_id: VarInt(i32::from(effect.status_effect().to_raw())),
        amplifier: VarInt(effect.amplifier()),
        duration: VarInt(effect.remaining_duration().unwrap_or(-1)),
        flags: update_mob_effect_s2c::Flags::new()
            .with_is_ambient(effect.ambient())
            .with_show_particles(effect.show_particles())
            .with_show_icon(effect.show_icon()),
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct StatusEffectQuery {
    entity: Entity,
    active_effects: &'static mut ActiveStatusEffects,
    client: Option<&'static mut Client>,
    entity_flags: Option<&'static mut Flags>,
    swirl_ambient: Option<&'static mut PotionSwirlsAmbient>,
}

fn add_status_effects(
    mut query: Query<StatusEffectQuery>,
    mut add_events: MessageWriter<StatusEffectAdded>,
    mut remove_events: MessageWriter<StatusEffectRemoved>,
) {
    for mut query in &mut query {
        let updated = query.active_effects.apply_changes();

        if updated.is_empty() {
            continue;
        }

        set_swirl(&query.active_effects, &mut query.swirl_ambient);

        for (status_effect, prev) in updated {
            if query.active_effects.has_effect(status_effect) {
                add_events.write(StatusEffectAdded {
                    entity: query.entity,
                    status_effect,
                });
            } else if let Some(prev) = prev {
                remove_events.write(StatusEffectRemoved {
                    entity: query.entity,
                    status_effect: prev,
                });
            } else {
                // this should never happen
                panic!("status effect was removed but was never added");
            }

            update_status_effect(&mut query, status_effect);
        }
    }
}

fn update_status_effect(query: &mut StatusEffectQueryItem, status_effect: StatusEffect) {
    let current_effect = query.active_effects.get_current_effect(status_effect);

    if let Some(ref mut client) = query.client {
        if let Some(updated_effect) = current_effect {
            client.write_packet(&create_packet(updated_effect));
        } else {
            client.write_packet(&RemoveMobEffectS2c {
                entity_id: VarInt(0),
                effect_id: VarInt(i32::from(status_effect.to_raw())),
            });
        }
    }
}

fn set_swirl(
    active_status_effects: &ActiveStatusEffects,
    swirl_ambient: &mut Option<Mut<'_, PotionSwirlsAmbient>>,
) {
    if let Some(ref mut swirl_ambient) = swirl_ambient {
        swirl_ambient.0 = active_status_effects
            .get_current_effects()
            .iter()
            .any(|effect| effect.ambient());
    }
}

/// Used to set the color of the swirls in the potion effect.
///
/// Equivalent to net.minecraft.component.type.PotionContentsComponent#mixColors
/// (Yarn mapping).
fn _get_color(effects: &ActiveStatusEffects) -> i32 {
    if effects.no_effects() {
        // vanilla mc seems to return 0xFF385DC6 (i32), decimal: -13083194 if there are
        // no effects dunno why
        // imma just say to return 0 to remove the swirls
        return 0;
    }

    let effects = effects.get_current_effects();
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    let mut total = 0;

    for status_effect_instance in effects {
        if !status_effect_instance.show_particles() {
            continue;
        }

        let color: u32 = status_effect_instance.status_effect().color();
        let weight = (status_effect_instance.amplifier() + 1) as u32;
        r += weight * ((color >> 16) & 0xff);
        g += weight * ((color >> 8) & 0xff);
        b += weight * ((color) & 0xff);
        total += weight;
    }

    if total == 0 {
        return 0;
    }

    let r = r / total;
    let g = g / total;
    let b = b / total;
    // Alpha is always 255
    ((0xff_u32 << 24) | (r << 16) | (g << 8) | b) as i32
}
