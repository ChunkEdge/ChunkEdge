package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.chunkedge.extractor.mixin.ExposeVoxelShape;
import com.chunkedge.extractor.mixin.ExposeWallBlock;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.tags.BlockTags;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.item.StandingAndWallBlockItem;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.phys.shapes.VoxelShape;

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
        var voxelShapes = new LinkedHashMap<VoxelShapeData, Integer>();

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
                stateJson.addProperty("opaque", state.canOcclude());
                stateJson.addProperty("replaceable", state.canBeReplaced());
                stateJson.addProperty("has_fluid", !state.getFluidState().isEmpty());
                stateJson.addProperty("blocks_motion", state.is(BlockTags.BLOCKS_MOTION_IN_HEIGHTMAP));
                stateJson.addProperty("blocks_motion_no_leaves", state.is(BlockTags.BLOCKS_MOTION_IN_HEIGHTMAP_NO_LEAVES));

                if (block.defaultBlockState().equals(state)) {
                    blockJson.addProperty("default_state_id", id);
                }

                var collisionShape = state.getCollisionShape(
                    EmptyBlockGetter.INSTANCE,
                    BlockPos.ZERO
                );

                var collisionShapeIdxsJson = new JsonArray();
                for (var box : collisionShape.toAabbs()) {
                    var shape = new Shape(
                        box.minX,
                        box.minY,
                        box.minZ,
                        box.maxX,
                        box.maxY,
                        box.maxZ
                    );

                    var idx = shapes.putIfAbsent(shape, shapes.size());
                    collisionShapeIdxsJson.add(
                        Objects.requireNonNullElseGet(
                            idx,
                            () -> shapes.size() - 1
                        )
                    );
                }

                stateJson.add("collision_shapes", collisionShapeIdxsJson);
                stateJson.addProperty(
                    "collision_voxel_shape",
                    voxelShapes.computeIfAbsent(
                        VoxelShapeData.of(collisionShape),
                        unused -> voxelShapes.size()
                    )
                );

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

        var voxelShapesJson = new JsonArray();
        for (var voxelShape : voxelShapes.keySet()) {
            voxelShapesJson.add(voxelShape.toJson());
        }

        topLevelJson.add("block_entity_types", blockEntitiesJson);
        topLevelJson.add("shapes", shapesJson);
        topLevelJson.add("voxel_shapes", voxelShapesJson);
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

    /**
     * A collision shape as vanilla's collision code sees it: the sorted
     * coordinate list of each axis (one more entry than there are cells on
     * that axis) and which cells of the grid are full. Collisions scan this
     * grid, so the internal grid lines matter and {@code toAabbs()} is not
     * enough.
     */
    private record VoxelShapeData(
        List<Double> x,
        List<Double> y,
        List<Double> z,
        String full
    ) {
        static VoxelShapeData of(VoxelShape shape) {
            var discrete = ((ExposeVoxelShape) shape).getDiscreteShape();
            var sizeX = discrete.getXSize();
            var sizeY = discrete.getYSize();
            var sizeZ = discrete.getZSize();

            // Same cell order as vanilla's BitSetDiscreteVoxelShape:
            // index = (x * sizeY + y) * sizeZ + z, least significant bit
            // first within each byte.
            var bytes = new byte[(sizeX * sizeY * sizeZ + 7) / 8];
            for (var x = 0; x < sizeX; x++) {
                for (var y = 0; y < sizeY; y++) {
                    for (var z = 0; z < sizeZ; z++) {
                        if (discrete.isFull(x, y, z)) {
                            var index = (x * sizeY + y) * sizeZ + z;
                            bytes[index / 8] |= (byte) (1 << (index % 8));
                        }
                    }
                }
            }

            var full = new StringBuilder();
            for (var b : bytes) {
                full.append(String.format("%02x", b & 0xff));
            }

            return new VoxelShapeData(
                coords(shape, Direction.Axis.X, sizeX),
                coords(shape, Direction.Axis.Y, sizeY),
                coords(shape, Direction.Axis.Z, sizeZ),
                full.toString()
            );
        }

        private static List<Double> coords(
            VoxelShape shape,
            Direction.Axis axis,
            int size
        ) {
            var coords = new ArrayList<>(shape.getCoords(axis));
            if (coords.size() != size + 1) {
                throw new IllegalStateException(
                    "expected " + (size + 1) + " " + axis + " coords, got " + coords
                );
            }
            return coords;
        }

        JsonObject toJson() {
            var json = new JsonObject();
            json.add("x", coordsJson(x));
            json.add("y", coordsJson(y));
            json.add("z", coordsJson(z));
            json.addProperty("full", full);
            return json;
        }

        private static JsonArray coordsJson(List<Double> coords) {
            var json = new JsonArray();
            coords.forEach(json::add);
            return json;
        }
    }
}
