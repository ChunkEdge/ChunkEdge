package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.ClassComparator;
import com.chunkedge.extractor.DummyPlayerEntity;
import com.chunkedge.extractor.LegacyNames;
import com.chunkedge.extractor.Main;
import com.google.gson.*;
import com.mojang.authlib.GameProfile;
import java.lang.reflect.ParameterizedType;
import java.util.*;
import net.minecraft.core.BlockPos;
import net.minecraft.core.GlobalPos;
import net.minecraft.core.Holder;
import net.minecraft.core.Registry;
import net.minecraft.core.Rotations;
import net.minecraft.core.particles.ParticleOptions;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.Component;
import net.minecraft.network.syncher.EntityDataAccessor;
import net.minecraft.network.syncher.EntityDataSerializers;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.*;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.DefaultAttributes;
import net.minecraft.world.entity.animal.armadillo.Armadillo;
import net.minecraft.world.entity.animal.sniffer.Sniffer;
import net.minecraft.world.entity.npc.villager.VillagerData;
import net.minecraft.world.entity.npc.villager.VillagerProfession;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.state.BlockState;
import org.jetbrains.annotations.Nullable;
import org.joml.Quaternionf;
import org.joml.Vector3f;

public class Entities implements Main.Extractor {

    private final ServerLevel world;
    private final Registry<VillagerProfession> professionRegistry;

    public Entities(MinecraftServer server) {
        this.world = server.overworld();
        this.professionRegistry = server
            .registryAccess()
            .lookupOrThrow(
                net.minecraft.core.registries.Registries.VILLAGER_PROFESSION
            );
    }

    private Main.Pair<String, JsonElement> trackedDataToJson(
        EntityDataAccessor<?> data,
        SynchedEntityData tracker
    ) {
        final var handler = data.serializer();
        final var val = tracker.get(data);

        if (handler == EntityDataSerializers.BYTE) {
            return new Main.Pair<>("byte", new JsonPrimitive((Byte) val));
        } else if (handler == EntityDataSerializers.INT) {
            return new Main.Pair<>("integer", new JsonPrimitive((Integer) val));
        } else if (handler == EntityDataSerializers.LONG) {
            return new Main.Pair<>("long", new JsonPrimitive((Long) val));
        } else if (handler == EntityDataSerializers.FLOAT) {
            return new Main.Pair<>("float", new JsonPrimitive((Float) val));
        } else if (handler == EntityDataSerializers.STRING) {
            return new Main.Pair<>("string", new JsonPrimitive((String) val));
        } else if (handler == EntityDataSerializers.COMPONENT) {
            // TODO: return text as json element.
            return new Main.Pair<>(
                "text_component",
                new JsonPrimitive(((Component) val).getString())
            );
        } else if (
            handler == EntityDataSerializers.OPTIONAL_COMPONENT
        ) {
            var res = ((Optional<?>) val)
                .map(o ->
                    (JsonElement) new JsonPrimitive(
                        ((Component) o).getString()
                    )
                )
                .orElse(JsonNull.INSTANCE);
            return new Main.Pair<>("optional_text_component", res);
        } else if (handler == EntityDataSerializers.ITEM_STACK) {
            return new Main.Pair<>(
                "item_stack",
                new JsonPrimitive(((ItemStack) val).toString())
            );
        } else if (handler == EntityDataSerializers.BOOLEAN) {
            return new Main.Pair<>(
                "boolean",
                new JsonPrimitive((Boolean) val)
            );
        } else if (handler == EntityDataSerializers.ROTATIONS) {
            var json = new JsonObject();
            var ea = (Rotations) val;
            json.addProperty("pitch", ea.x());
            json.addProperty("yaw", ea.y());
            json.addProperty("roll", ea.z());
            return new Main.Pair<>("rotation", json);
        } else if (handler == EntityDataSerializers.BLOCK_POS) {
            var bp = (BlockPos) val;
            var json = new JsonObject();
            json.addProperty("x", bp.getX());
            json.addProperty("y", bp.getY());
            json.addProperty("z", bp.getZ());
            return new Main.Pair<>("block_pos", json);
        } else if (handler == EntityDataSerializers.OPTIONAL_BLOCK_POS) {
            return new Main.Pair<>(
                "optional_block_pos",
                ((Optional<?>) val)
                    .map(o -> {
                        var bp = (BlockPos) o;
                        var json = new JsonObject();
                        json.addProperty("x", bp.getX());
                        json.addProperty("y", bp.getY());
                        json.addProperty("z", bp.getZ());
                        return (JsonElement) json;
                    })
                    .orElse(JsonNull.INSTANCE)
            );
        } else if (handler == EntityDataSerializers.DIRECTION) {
            return new Main.Pair<>(
                "facing",
                new JsonPrimitive(val.toString())
            );
        } else if (handler == EntityDataSerializers.BLOCK_STATE) {
            // TODO: get raw block state ID.
            var state = (BlockState) val;
            return new Main.Pair<>(
                "block_state",
                new JsonPrimitive(state.toString())
            );
        } else if (handler == EntityDataSerializers.OPTIONAL_BLOCK_STATE) {
            // TODO: get raw block state ID.
            var res = ((Optional<?>) val)
                .map(o -> (JsonElement) new JsonPrimitive(o.toString()))
                .orElse(JsonNull.INSTANCE);
            return new Main.Pair<>("optional_block_state", res);
        } else if (handler == EntityDataSerializers.PARTICLE) {
            var id = BuiltInRegistries.PARTICLE_TYPE.getKey(
                ((ParticleOptions) val).getType()
            );
            return new Main.Pair<>(
                "particle",
                new JsonPrimitive(id.getPath())
            );
        } else if (handler == EntityDataSerializers.PARTICLES) {
            @SuppressWarnings("unchecked")
            List<ParticleOptions> particleList = (List<ParticleOptions>) val;
            JsonArray json = new JsonArray();
            for (ParticleOptions particleEffect : particleList) {
                var id = BuiltInRegistries.PARTICLE_TYPE.getKey(
                    particleEffect.getType()
                );
                json.add(new JsonPrimitive(id.getPath()));
            }
            return new Main.Pair<>("particle_list", json);
        } else if (handler == EntityDataSerializers.VILLAGER_DATA) {
            var vd = (VillagerData) val;
            var json = new JsonObject();
            var type = BuiltInRegistries.VILLAGER_TYPE.getKey(
                vd.type().value()
            ).getPath();
            var profession = professionRegistry
                .getKey(vd.profession().value())
                .getPath();
            json.addProperty("type", type);
            json.addProperty("profession", profession);
            json.addProperty("level", vd.level());
            return new Main.Pair<>("villager_data", json);
        } else if (handler == EntityDataSerializers.OPTIONAL_UNSIGNED_INT) {
            var opt = (OptionalInt) val;
            return new Main.Pair<>(
                "optional_int",
                opt.isPresent()
                    ? new JsonPrimitive(opt.getAsInt())
                    : JsonNull.INSTANCE
            );
        } else if (handler == EntityDataSerializers.POSE) {
            return new Main.Pair<>(
                "entity_pose",
                new JsonPrimitive(
                    ((Pose) val).name().toLowerCase(Locale.ROOT)
                )
            );
        } else if (
            handler == EntityDataSerializers.CAT_VARIANT ||
            handler == EntityDataSerializers.CAT_SOUND_VARIANT ||
            handler == EntityDataSerializers.WOLF_SOUND_VARIANT ||
            handler == EntityDataSerializers.WOLF_VARIANT ||
            handler == EntityDataSerializers.FROG_VARIANT ||
            handler == EntityDataSerializers.COW_VARIANT ||
            handler == EntityDataSerializers.COW_SOUND_VARIANT ||
            handler == EntityDataSerializers.CHICKEN_VARIANT ||
            handler == EntityDataSerializers.CHICKEN_SOUND_VARIANT ||
            handler == EntityDataSerializers.PIG_VARIANT ||
            handler == EntityDataSerializers.PIG_SOUND_VARIANT ||
            handler == EntityDataSerializers.ZOMBIE_NAUTILUS_VARIANT
        ) {
            return new Main.Pair<>(
                variantJsonKey(handler),
                new JsonPrimitive(holderIdString((Holder<?>) val))
            );
        } else if (handler == EntityDataSerializers.OPTIONAL_GLOBAL_POS) {
            return new Main.Pair<>(
                "optional_global_pos",
                ((Optional<?>) val)
                    .map(o -> {
                        var gp = (GlobalPos) o;
                        var json = new JsonObject();
                        json.addProperty(
                            "dimension",
                            gp.dimension().identifier().toString()
                        );

                        var posJson = new JsonObject();
                        posJson.addProperty("x", gp.pos().getX());
                        posJson.addProperty("y", gp.pos().getY());
                        posJson.addProperty("z", gp.pos().getZ());

                        json.add("position", posJson);
                        return (JsonElement) json;
                    })
                    .orElse(JsonNull.INSTANCE)
            );
        } else if (handler == EntityDataSerializers.PAINTING_VARIANT) {
            var variant = ((Holder<?>) val).unwrapKey()
                .map(k -> k.identifier().getPath())
                .orElse("");
            return new Main.Pair<>(
                "painting_variant",
                new JsonPrimitive(variant)
            );
        } else if (handler == EntityDataSerializers.SNIFFER_STATE) {
            return new Main.Pair<>(
                "sniffer_state",
                new JsonPrimitive(
                    ((Sniffer.State) val).name().toLowerCase(Locale.ROOT)
                )
            );
        } else if (handler == EntityDataSerializers.ARMADILLO_STATE) {
            return new Main.Pair<>(
                "armadillo_state",
                new JsonPrimitive(
                    ((Armadillo.ArmadilloState) val).name()
                        .toLowerCase(Locale.ROOT)
                )
            );
        } else if (handler == EntityDataSerializers.WEATHERING_COPPER_STATE) {
            return new Main.Pair<>(
                "weathering_copper_state",
                new JsonPrimitive(
                    ((Enum<?>) val).name().toLowerCase(Locale.ROOT)
                )
            );
        } else if (handler == EntityDataSerializers.COPPER_GOLEM_STATE) {
            return new Main.Pair<>(
                "copper_golem_state",
                new JsonPrimitive(
                    ((Enum<?>) val).name().toLowerCase(Locale.ROOT)
                )
            );
        } else if (handler == EntityDataSerializers.HUMANOID_ARM) {
            return new Main.Pair<>(
                "humanoid_arm",
                new JsonPrimitive(
                    ((Enum<?>) val).name().toLowerCase(Locale.ROOT)
                )
            );
        } else if (handler == EntityDataSerializers.RESOLVABLE_PROFILE) {
            return new Main.Pair<>(
                "resolvable_profile",
                new JsonPrimitive(val.toString())
            );
        } else if (handler == EntityDataSerializers.VECTOR3) {
            final Vector3f vec;
            if (val instanceof Vector3f v) {
                vec = v;
            } else {
                var fc = (org.joml.Vector3fc) val;
                vec = new Vector3f(fc);
            }
            var json = new JsonObject();
            json.addProperty("x", vec.x);
            json.addProperty("y", vec.y);
            json.addProperty("z", vec.z);
            return new Main.Pair<>("vector3f", json);
        } else if (handler == EntityDataSerializers.QUATERNION) {
            final Quaternionf quat;
            if (val instanceof Quaternionf q) {
                quat = q;
            } else {
                var fc = (org.joml.Quaternionfc) val;
                quat = new Quaternionf(fc);
            }
            var json = new JsonObject();
            json.addProperty("x", quat.x);
            json.addProperty("y", quat.y);
            json.addProperty("z", quat.z);
            json.addProperty("w", quat.w);
            return new Main.Pair<>("quaternionf", json);
        } else if (
            handler == EntityDataSerializers.OPTIONAL_LIVING_ENTITY_REFERENCE
        ) {
            return new Main.Pair<>(
                "lazy_entity_reference",
                ((Optional<?>) val)
                    .map(o -> (JsonElement) new JsonObject())
                    .orElse(JsonNull.INSTANCE)
            );
        } else {
            throw new IllegalArgumentException(
                "Unexpected tracked handler of ID " +
                    EntityDataSerializers.getSerializedId(handler) +
                    handler.toString()
            );
        }
    }

    private static String variantJsonKey(Object handler) {
        if (handler == EntityDataSerializers.CAT_VARIANT) {
            return "cat_variant";
        } else if (handler == EntityDataSerializers.CAT_SOUND_VARIANT) {
            return "cat_sound_variant";
        } else if (handler == EntityDataSerializers.WOLF_SOUND_VARIANT) {
            return "wolf_sound_variant";
        } else if (handler == EntityDataSerializers.WOLF_VARIANT) {
            return "wolf_variant";
        } else if (handler == EntityDataSerializers.FROG_VARIANT) {
            return "frog_variant";
        } else if (handler == EntityDataSerializers.COW_VARIANT) {
            return "cow_variant";
        } else if (handler == EntityDataSerializers.COW_SOUND_VARIANT) {
            return "cow_sound_variant";
        } else if (handler == EntityDataSerializers.CHICKEN_VARIANT) {
            return "chicken_variant";
        } else if (handler == EntityDataSerializers.CHICKEN_SOUND_VARIANT) {
            return "chicken_sound_variant";
        } else if (handler == EntityDataSerializers.PIG_VARIANT) {
            return "pig_variant";
        } else if (handler == EntityDataSerializers.PIG_SOUND_VARIANT) {
            return "pig_sound_variant";
        } else {
            return "zombie_nautilus_variant";
        }
    }

    private static String holderIdString(Holder<?> holder) {
        return holder.unwrapKey().map(k -> k.identifier().toString()).orElse("");
    }

    @Override
    public String fileName() {
        return "entities.json";
    }

    @Override
    @SuppressWarnings("unchecked")
    public JsonElement extract()
        throws IllegalAccessException, NoSuchFieldException {
        final var entityList = new ArrayList<
            Main.Pair<Class<? extends Entity>, EntityType<?>>
        >();
        var entityClassTypeMap = new HashMap<
            Class<? extends Entity>,
            EntityType<?>
        >();
        for (var f : EntityTypes.class.getFields()) {
            if (f.getType().equals(EntityType.class)) {
                var entityClass = (Class<? extends Entity>) ((ParameterizedType) f.getGenericType())
                    .getActualTypeArguments()[0];
                var entityType = (EntityType<?>) f.get(null);

                entityList.add(new Main.Pair<>(entityClass, entityType));
                entityClassTypeMap.put(entityClass, entityType);
            }
        }

        final var dataTrackerField = Entity.class.getDeclaredField(
            "entityData"
        );
        dataTrackerField.setAccessible(true);

        var entitiesMap = new TreeMap<Class<? extends Entity>, JsonElement>(
            new ClassComparator()
        );

        for (var entry : entityList) {
            var entityClass = entry.left();
            @Nullable
            var entityType = entry.right();
            assert null != entityType;

            // While we can use the tracked data registry and reflection to get the tracked
            // fields on entities, we won't know what their default values are because they
            // are assigned in the entity's constructor.
            // To obtain this, we create a dummy world to spawn the entities into and read
            // the data tracker field from the base entity class.
            // We also handle player entities specially since they cannot be spawned with
            // EntityType#create.
            final var entityInstance = entityType.equals(EntityTypes.PLAYER)
                ? new DummyPlayerEntity(world, new GameProfile(UUID.randomUUID(), "cooldude"))
                : entityType.create(world, EntitySpawnReason.COMMAND);

            final var dataTracker = (SynchedEntityData) dataTrackerField.get(
                entityInstance
            );

            while (null == entitiesMap.get(entityClass)) {
                var entityJson = new JsonObject();

                var parent = entityClass.getSuperclass();
                var hasParent =
                    null != parent && Entity.class.isAssignableFrom(parent);

                if (hasParent) {
                    entityJson.addProperty(
                        "parent",
                        LegacyNames.className(parent.getSimpleName())
                    );
                }

                if (null != entityType) {
                    entityJson.addProperty(
                        "type",
                        BuiltInRegistries.ENTITY_TYPE.getKey(
                            entityType
                        ).getPath()
                    );

                    entityJson.add(
                        "translation_key",
                        new JsonPrimitive(entityType.getDescriptionId())
                    );
                }

                var fieldsJson = new JsonArray();
                for (var entityField : entityClass.getDeclaredFields()) {
                    if (
                        entityField.getType().equals(EntityDataAccessor.class)
                    ) {
                        entityField.setAccessible(true);

                        var trackedData = (EntityDataAccessor<?>) entityField.get(
                            null
                        );

                        var fieldJson = new JsonObject();
                        var officialFieldName = entityField
                            .getName()
                            .toLowerCase(Locale.ROOT);
                        // Keep the legacy (1.21.5-era) JSON keys so the
                        // generated Rust API stays stable across versions.
                        var ownerName = entityClass.getSimpleName();
                        var fieldName = LegacyNames.fieldName(
                            ownerName,
                            officialFieldName
                        );
                        fieldJson.addProperty("name", fieldName);
                        fieldJson.addProperty("index", trackedData.id());

                        var data = trackedDataToJson(trackedData, dataTracker);
                        var legacyType = LegacyNames.fieldTypeOrNull(
                            ownerName,
                            officialFieldName
                        );
                        fieldJson.addProperty(
                            "type",
                            legacyType != null ? legacyType : data.left()
                        );
                        fieldJson.add("default_value", data.right());

                        fieldsJson.add(fieldJson);
                    }
                }
                entityJson.add("fields", fieldsJson);

                if (entityInstance instanceof LivingEntity) {
                    var type = (EntityType<? extends LivingEntity>) entityType;
                    var defaultAttributes = DefaultAttributes.getSupplier(
                        type
                    );
                    var attributesJson = new JsonArray();
                    if (null != defaultAttributes) {
                        var instancesField = defaultAttributes
                            .getClass()
                            .getDeclaredField("instances");
                        instancesField.setAccessible(true);
                        var instances = (Map<
                            Holder<Attribute>,
                            AttributeInstance
                        >) instancesField.get(defaultAttributes);

                        for (var instance : instances.values()) {
                            var attribute = instance.getAttribute().value();

                            var attributeJson = new JsonObject();

                            attributeJson.addProperty(
                                "id",
                                BuiltInRegistries.ATTRIBUTE.getId(attribute)
                            );
                            attributeJson.addProperty(
                                "name",
                                BuiltInRegistries.ATTRIBUTE.getKey(
                                    attribute
                                ).getPath()
                            );
                            attributeJson.addProperty(
                                "base_value",
                                instance.getBaseValue()
                            );

                            attributesJson.add(attributeJson);
                        }
                    }
                    entityJson.add("attributes", attributesJson);
                }

                var bb = entityInstance.getBoundingBox();
                if (null != bb && null != entityType) {
                    var boundingBoxJson = new JsonObject();

                    boundingBoxJson.addProperty("size_x", bb.getXsize());
                    boundingBoxJson.addProperty("size_y", bb.getYsize());
                    boundingBoxJson.addProperty("size_z", bb.getZsize());

                    entityJson.add("default_bounding_box", boundingBoxJson);
                }

                entitiesMap.put(entityClass, entityJson);

                if (!hasParent) {
                    break;
                }

                entityClass = (Class<? extends Entity>) parent;
                entityType = entityClassTypeMap.get(entityClass);
            }
        }

        var entitiesJson = new JsonObject();
        for (var entry : entitiesMap.entrySet()) {
            entitiesJson.add(
                LegacyNames.className(entry.getKey().getSimpleName()),
                entry.getValue()
            );
        }

        return entitiesJson;
    }
}
