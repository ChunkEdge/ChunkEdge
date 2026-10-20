package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.chunkedge.extractor.mixin.ExposeWallBlock;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Objects;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.item.StandingAndWallBlockItem;
import net.minecraft.world.level.EmptyBlockGetter;

public class Blocks implements Main.Extractor {

    public Blocks() {}

    @Override
    public String fileName() {
        return "blocks.json";
    }

    @Override
    @SuppressWarnings("deprecation")
    public JsonElement extract() {
        var topLevelJson = new JsonObject();

        var blocksJson = new JsonArray();
        var stateIdCounter = 0;

        var shapes = new LinkedHashMap<Shape, Integer>();

        for (var block : BuiltInRegistries.BLOCK) {
            var blockJson = new JsonObject();
            blockJson.addProperty(
                "id",
                BuiltInRegistries.BLOCK.getId(block)
            );
            blockJson.addProperty(
                "name",
                BuiltInRegistries.BLOCK.getKey(block).getPath()
            );
            blockJson.addProperty(
                "translation_key",
                block.getDescriptionId()
            );
            blockJson.addProperty(
                "item_id",
                BuiltInRegistries.ITEM.getId(block.asItem())
            );

            if (
                block.asItem() instanceof StandingAndWallBlockItem wsbItem
            ) {
                if (wsbItem.getBlock() == block) {
                    var wallBlock = ((ExposeWallBlock) wsbItem).getWallBlock();
                    blockJson.addProperty(
                        "wall_variant_id",
                        BuiltInRegistries.BLOCK.getId(wallBlock)
                    );
                }
            }

            var propsJson = new JsonArray();
            for (var prop : block.getStateDefinition().getProperties()) {
                var propJson = new JsonObject();

                propJson.addProperty("name", prop.getName());

                var valuesJson = new JsonArray();
                for (var value : prop.getPossibleValues()) {
                    valuesJson.add(value.toString().toLowerCase(Locale.ROOT));
                }
                propJson.add("values", valuesJson);

                propsJson.add(propJson);
            }
            blockJson.add("properties", propsJson);

            var statesJson = new JsonArray();
            for (var state : block.getStateDefinition().getPossibleStates()) {
                var stateJson = new JsonObject();
                var id = stateIdCounter;
                stateIdCounter++;
                stateJson.addProperty("id", id);
                stateJson.addProperty("luminance", state.getLightEmission());
                stateJson.addProperty("opaque", state.isSolid());
                stateJson.addProperty("replaceable", state.canBeReplaced());
                // This uses a deprecated api, but minecraft uses the same deprecated api, so we use it for now
                stateJson.addProperty("blocks_motion", state.blocksMotion());

                if (block.defaultBlockState().equals(state)) {
                    blockJson.addProperty("default_state_id", id);
                }

                var collisionShapeIdxsJson = new JsonArray();
                for (var box : state
                    .getCollisionShape(
                        EmptyBlockGetter.INSTANCE,
                        BlockPos.ZERO
                    )
                    .toAabbs()) {
                    var collisionShape = new Shape(
                        box.minX,
                        box.minY,
                        box.minZ,
                        box.maxX,
                        box.maxY,
                        box.maxZ
                    );

                    var idx = shapes.putIfAbsent(collisionShape, shapes.size());
                    collisionShapeIdxsJson.add(
                        Objects.requireNonNullElseGet(
                            idx,
                            () -> shapes.size() - 1
                        )
                    );
                }

                stateJson.add("collision_shapes", collisionShapeIdxsJson);

                for (
                    var blockEntity : BuiltInRegistries.BLOCK_ENTITY_TYPE
                ) {
                    if (blockEntity.isValid(state)) {
                        stateJson.addProperty(
                            "block_entity_type",
                            BuiltInRegistries.BLOCK_ENTITY_TYPE.getId(
                                blockEntity
                            )
                        );
                    }
                }

                statesJson.add(stateJson);
            }
            blockJson.add("states", statesJson);

            blocksJson.add(blockJson);
        }

        var blockEntitiesJson = new JsonArray();
        for (var blockEntity : BuiltInRegistries.BLOCK_ENTITY_TYPE) {
            var blockEntityJson = new JsonObject();
            blockEntityJson.addProperty(
                "id",
                BuiltInRegistries.BLOCK_ENTITY_TYPE.getId(blockEntity)
            );
            blockEntityJson.addProperty(
                "ident",
                BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(
                    blockEntity
                ).toString()
            );
            blockEntityJson.addProperty(
                "name",
                BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(
                    blockEntity
                ).getPath()
            );

            blockEntitiesJson.add(blockEntityJson);
        }

        var shapesJson = new JsonArray();
        for (var shape : shapes.keySet()) {
            var shapeJson = new JsonObject();
            shapeJson.addProperty("min_x", shape.minX);
            shapeJson.addProperty("min_y", shape.minY);
            shapeJson.addProperty("min_z", shape.minZ);
            shapeJson.addProperty("max_x", shape.maxX);
            shapeJson.addProperty("max_y", shape.maxY);
            shapeJson.addProperty("max_z", shape.maxZ);
            shapesJson.add(shapeJson);
        }

        topLevelJson.add("block_entity_types", blockEntitiesJson);
        topLevelJson.add("shapes", shapesJson);
        topLevelJson.add("blocks", blocksJson);

        return topLevelJson;
    }

    private record Shape(
        double minX,
        double minY,
        double minZ,
        double maxX,
        double maxY,
        double maxZ
    ) {}
}
