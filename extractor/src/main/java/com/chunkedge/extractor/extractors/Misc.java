package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.chunkedge.extractor.LegacyNames;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.lang.reflect.Modifier;
import java.util.Locale;
import net.minecraft.core.Direction;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.protocol.game.ClientboundAnimatePacket;
import net.minecraft.network.syncher.EntityDataSerializer;
import net.minecraft.network.syncher.EntityDataSerializers;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.entity.EntityEvent;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.animal.armadillo.Armadillo;
import net.minecraft.world.entity.animal.sniffer.Sniffer;

public class Misc implements Main.Extractor {

    private final MinecraftServer server;

    public Misc(MinecraftServer server) {
        this.server = server;
    }

    @Override
    public String fileName() {
        return "misc.json";
    }

    private <T> Registry<T> dynamicRegistry(
        ResourceKey<Registry<T>> key
    ) {
        return server.registryAccess().lookupOrThrow(key);
    }

    @Override
    public JsonElement extract() throws Exception {
        var miscJson = new JsonObject();

        var entityTypeJson = new JsonObject();
        for (var type : BuiltInRegistries.ENTITY_TYPE) {
            entityTypeJson.addProperty(
                BuiltInRegistries.ENTITY_TYPE.getKey(type).getPath(),
                BuiltInRegistries.ENTITY_TYPE.getId(type)
            );
        }
        miscJson.add("entity_type", entityTypeJson);

        var entityStatusJson = new JsonObject();
        for (var field : EntityEvent.class.getDeclaredFields()) {
            if (
                Modifier.isStatic(field.getModifiers()) &&
                field.canAccess(null) &&
                field.get(null) instanceof Byte code
            ) {
                entityStatusJson.addProperty(
                    LegacyNames.statusName(
                        field.getName().toLowerCase(Locale.ROOT)
                    ),
                    code
                );
            }
        }
        miscJson.add("entity_status", entityStatusJson);

        var entityAnimationJson = new JsonObject();
        for (var field : ClientboundAnimatePacket.class.getDeclaredFields()) {
            field.setAccessible(true);
            if (
                Modifier.isStatic(field.getModifiers()) &&
                field.canAccess(null) &&
                field.get(null) instanceof Integer i
            ) {
                entityAnimationJson.addProperty(
                    LegacyNames.animationName(
                        field.getName().toLowerCase(Locale.ROOT)
                    ),
                    i
                );
            }
        }
        miscJson.add("entity_animation", entityAnimationJson);

        var villagerTypeJson = new JsonObject();
        for (var type : BuiltInRegistries.VILLAGER_TYPE) {
            villagerTypeJson.addProperty(
                BuiltInRegistries.VILLAGER_TYPE.getKey(type).getPath(),
                BuiltInRegistries.VILLAGER_TYPE.getId(type)
            );
        }
        miscJson.add("villager_type", villagerTypeJson);

        var villagerProfessionJson = new JsonObject();
        var professionRegistry = dynamicRegistry(
            Registries.VILLAGER_PROFESSION
        );
        for (var profession : professionRegistry) {
            villagerProfessionJson.addProperty(
                professionRegistry
                    .getKey(profession)
                    .toString()
                    .toLowerCase(Locale.ROOT),
                professionRegistry.getId(profession)
            );
        }
        miscJson.add("villager_profession", villagerProfessionJson);

        miscJson.add(
            "cat_variant",
            dynamicVariants(dynamicRegistry(Registries.CAT_VARIANT))
        );
        miscJson.add(
            "frog_variant",
            dynamicVariants(dynamicRegistry(Registries.FROG_VARIANT))
        );
        miscJson.add(
            "wolf_variant",
            dynamicVariants(dynamicRegistry(Registries.WOLF_VARIANT))
        );
        miscJson.add(
            "pig_variant",
            dynamicVariants(dynamicRegistry(Registries.PIG_VARIANT))
        );
        miscJson.add(
            "cow_variant",
            dynamicVariants(dynamicRegistry(Registries.COW_VARIANT))
        );
        miscJson.add(
            "chicken_variant",
            dynamicVariants(dynamicRegistry(Registries.CHICKEN_VARIANT))
        );
        miscJson.add(
            "painting_variant",
            dynamicVariants(dynamicRegistry(Registries.PAINTING_VARIANT))
        );
        miscJson.add(
            "wolf_sound_variant",
            dynamicVariants(dynamicRegistry(Registries.WOLF_SOUND_VARIANT))
        );
        miscJson.add(
            "cat_sound_variant",
            dynamicVariants(dynamicRegistry(Registries.CAT_SOUND_VARIANT))
        );
        miscJson.add(
            "chicken_sound_variant",
            dynamicVariants(
                dynamicRegistry(Registries.CHICKEN_SOUND_VARIANT)
            )
        );
        miscJson.add(
            "cow_sound_variant",
            dynamicVariants(dynamicRegistry(Registries.COW_SOUND_VARIANT))
        );
        miscJson.add(
            "pig_sound_variant",
            dynamicVariants(dynamicRegistry(Registries.PIG_SOUND_VARIANT))
        );
        miscJson.add(
            "zombie_nautilus_variant",
            dynamicVariants(
                dynamicRegistry(Registries.ZOMBIE_NAUTILUS_VARIANT)
            )
        );

        var directionJson = new JsonObject();
        for (var dir : Direction.values()) {
            directionJson.addProperty(dir.name(), dir.get3DDataValue());
        }
        miscJson.add("direction", directionJson);

        var entityPoseJson = new JsonObject();
        var poses = Pose.values();
        for (int i = 0; i < poses.length; i++) {
            entityPoseJson.addProperty(
                poses[i].name().toLowerCase(Locale.ROOT),
                i
            );
        }
        miscJson.add("entity_pose", entityPoseJson);

        var particleTypesJson = new JsonObject();
        for (var type : BuiltInRegistries.PARTICLE_TYPE) {
            particleTypesJson.addProperty(
                BuiltInRegistries.PARTICLE_TYPE.getKey(type).getPath(),
                BuiltInRegistries.PARTICLE_TYPE.getId(type)
            );
        }
        miscJson.add("particle_type", particleTypesJson);

        var snifferStateJson = new JsonObject();
        for (var state : Sniffer.State.values()) {
            snifferStateJson.addProperty(
                state.name().toLowerCase(Locale.ROOT),
                state.ordinal()
            );
        }
        miscJson.add("sniffer_state", snifferStateJson);

        var armadilloStateJson = new JsonObject();
        for (var state : Armadillo.ArmadilloState.values()) {
            armadilloStateJson.addProperty(
                state.name().toLowerCase(Locale.ROOT),
                state.ordinal()
            );
        }
        miscJson.add("armadillo_state", armadilloStateJson);

        var trackedDataHandlerJson = new JsonObject();
        for (
            var field : EntityDataSerializers.class.getDeclaredFields()
        ) {
            field.setAccessible(true);
            if (
                Modifier.isStatic(field.getModifiers()) &&
                field.get(null) instanceof EntityDataSerializer<?> handler
            ) {
                var name = field.getName().toLowerCase(Locale.ROOT);
                var id = EntityDataSerializers.getSerializedId(handler);

                trackedDataHandlerJson.addProperty(name, id);
            }
        }
        miscJson.add("tracked_data_handler", trackedDataHandlerJson);

        return miscJson;
    }

    private static <T> JsonObject dynamicVariants(Registry<T> registry) {
        var json = new JsonObject();
        for (var variant : registry) {
            json.addProperty(
                registry.getKey(variant).getPath(),
                registry.getId(variant)
            );
        }
        return json;
    }
}
