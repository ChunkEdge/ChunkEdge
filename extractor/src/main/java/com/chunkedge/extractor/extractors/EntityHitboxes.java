package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.DummyPlayerEntity;
import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.authlib.GameProfile;
import java.util.Locale;
import java.util.UUID;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.syncher.EntityDataAccessor;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.*;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.entity.animal.fish.Pufferfish;
import net.minecraft.world.entity.animal.fish.Salmon;
import net.minecraft.world.entity.animal.sniffer.Sniffer;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.entity.monster.Phantom;
import net.minecraft.world.entity.monster.Zoglin;
import net.minecraft.world.entity.monster.cubemob.AbstractCubeMob;
import net.minecraft.world.entity.monster.piglin.Piglin;
import net.minecraft.world.entity.monster.zombie.Zombie;

/** Vanilla local bounding boxes, including age, pose, and discrete metadata variants. */
public final class EntityHitboxes implements Main.Extractor {
    private final ServerLevel world;

    public EntityHitboxes(MinecraftServer server) {
        world = server.overworld();
    }

    public String fileName() {
        return "entity_hitboxes.json";
    }

    public JsonElement extract() throws Exception {
        var result = new JsonObject();
        for (var type : BuiltInRegistries.ENTITY_TYPE) {
            Entity entity = type == EntityTypes.PLAYER
                ? new DummyPlayerEntity(world, new GameProfile(new UUID(0L, 0L), "hitbox"))
                : type.create(world, EntitySpawnReason.COMMAND);
            if (entity == null) {
                throw new IllegalStateException("Cannot create entity for hitbox extraction: " + type);
            }
            if (entity instanceof LivingEntity living) {
                var scale = living.getAttribute(Attributes.SCALE);
                if (scale != null) scale.setBaseValue(1.0);
            }
            // Cube dimensions depend on size, not age. Runtime multiplies this
            // size-one template by the synchronized cube size.
            if (entity instanceof AbstractCubeMob cube) cube.setSize(1, false);
            int variantCount = switch (entity) {
                case Pufferfish ignored -> 3;
                case Salmon ignored -> 3;
                case Phantom ignored -> 65;
                case Sniffer ignored -> 2;
                case ArmorStand ignored -> 4;
                default -> 1;
            };
            var variants = new JsonArray();
            for (int variant = 0; variant < variantCount; variant++) {
                switch (entity) {
                    case Pufferfish fish -> fish.setPuffState(variant);
                    case Salmon fish -> setData(fish, Salmon.class, "DATA_TYPE", variant);
                    case Phantom phantom -> phantom.setPhantomSize(variant);
                    case Sniffer sniffer -> setData(sniffer, Sniffer.class, "DATA_STATE",
                        variant == 0 ? Sniffer.State.IDLING : Sniffer.State.DIGGING);
                    case ArmorStand stand -> setData(stand, ArmorStand.class, "DATA_CLIENT_FLAGS",
                        (byte) ((variant & 1) | ((variant & 2) << 3)));
                    default -> {}
                }
                var sample = new JsonObject();
                setBaby(entity, false);
                var adult = poses(entity);
                setBaby(entity, true);
                var baby = poses(entity);
                sample.add("adult", adult);
                if (!adult.equals(baby)) sample.add("baby", baby);
                variants.add(sample);
            }
            result.add(BuiltInRegistries.ENTITY_TYPE.getKey(type).getPath(), variants);
        }
        return result;
    }

    private static void setBaby(Entity entity, boolean baby) {
        if (entity instanceof AbstractCubeMob) return;
        if (entity instanceof AgeableMob ageable) ageable.setBaby(baby);
        else if (entity instanceof Zombie zombie) zombie.setBaby(baby);
        else if (entity instanceof Piglin piglin) piglin.setBaby(baby);
        else if (entity instanceof Zoglin zoglin) zoglin.setBaby(baby);
    }

    @SuppressWarnings("unchecked")
    private static <T> void setData(Entity entity, Class<?> owner, String name, T value)
        throws ReflectiveOperationException {
        var field = owner.getDeclaredField(name);
        field.setAccessible(true);
        entity.getEntityData().set((EntityDataAccessor<T>) field.get(null), value);
    }

    private static JsonObject poses(Entity entity) {
        var result = new JsonObject();
        var standing = bounds(entity, Pose.STANDING);
        result.add("standing", standing);
        for (var pose : Pose.values()) {
            if (pose == Pose.STANDING) continue;
            var bounds = bounds(entity, pose);
            if (!bounds.equals(standing)) result.add(pose.name().toLowerCase(Locale.ROOT), bounds);
        }
        return result;
    }

    private static JsonObject bounds(Entity entity, Pose pose) {
        entity.setPose(pose);
        entity.refreshDimensions();
        var box = entity.getBoundingBox();
        var position = entity.position();
        var result = new JsonObject();
        var min = new JsonArray();
        min.add(box.minX - position.x);
        min.add(box.minY - position.y);
        min.add(box.minZ - position.z);
        var max = new JsonArray();
        max.add(box.maxX - position.x);
        max.add(box.maxY - position.y);
        max.add(box.maxZ - position.z);
        result.add("min", min);
        result.add("max", max);
        result.addProperty("scalable", !entity.getDimensions(pose).fixed());
        return result;
    }
}
