package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Comparator;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockBehaviour;

/**
 * Per-block values that drive entity movement: friction, speed and jump
 * factors, bounce restitution and the random per-position offset.
 *
 * <p>Floats are written with {@link Float#toString}, which prints the shortest
 * decimal that round-trips, so the same float always gives the same text and
 * consumers recover the exact bits by parsing that text as a 32-bit float.
 * Blocks are ordered by registry id and offset samples by a fixed position
 * list, so the output is deterministic.
 */
public class BlockPhysics implements Main.Extractor {

    /**
     * Positions at which every offset block's {@code getOffset} is sampled, so
     * ports of the offset function can be checked against vanilla.
     */
    private static final BlockPos[] OFFSET_SAMPLE_POSITIONS = {
        new BlockPos(0, 0, 0),
        new BlockPos(1, 0, 2),
        new BlockPos(-7, 0, 13),
        new BlockPos(100000, 0, -3),
        new BlockPos(-1, 5, -1),
        new BlockPos(12345, -60, -54321),
        new BlockPos(-29999999, 319, 29999999),
        new BlockPos(29999999, -64, -29999999),
        new BlockPos(686, 70, -1337),
        new BlockPos(-4096, 12, 4095),
    };

    private static final int OFFSET_TYPE_PROBES = 256;

    public BlockPhysics() {}

    @Override
    public String fileName() {
        return "block_physics.json";
    }

    @Override
    public JsonElement extract() throws ReflectiveOperationException {
        var maxHorizontalOffset = BlockBehaviour.class.getDeclaredMethod("getMaxHorizontalOffset");
        var maxVerticalOffset = BlockBehaviour.class.getDeclaredMethod("getMaxVerticalOffset");
        maxHorizontalOffset.setAccessible(true);
        maxVerticalOffset.setAccessible(true);

        var blocks = new ArrayList<Block>();
        BuiltInRegistries.BLOCK.forEach(blocks::add);
        blocks.sort(Comparator.comparingInt(BuiltInRegistries.BLOCK::getId));

        var blocksJson = new JsonArray();
        for (var block : blocks) {
            var blockJson = new JsonObject();
            blockJson.addProperty("id", BuiltInRegistries.BLOCK.getId(block));
            blockJson.addProperty("name", BuiltInRegistries.BLOCK.getKey(block).getPath());
            blockJson.addProperty("friction", block.getFriction());
            blockJson.addProperty("speed_factor", block.getSpeedFactor());
            blockJson.addProperty("jump_factor", block.getJumpFactor());
            blockJson.addProperty("bounce_restitution", block.getBounceRestitution());
            blockJson.add("offset", offset(block, maxHorizontalOffset, maxVerticalOffset));
            blocksJson.add(blockJson);
        }

        var topLevelJson = new JsonObject();
        topLevelJson.add("blocks", blocksJson);
        return topLevelJson;
    }

    private static JsonElement offset(Block block, Method maxHorizontalOffset, Method maxVerticalOffset)
        throws ReflectiveOperationException {
        // The offset function comes from the block's properties, so every state shares it.
        for (var state : block.getStateDefinition().getPossibleStates()) {
            if (state.hasOffsetFunction() != block.defaultBlockState().hasOffsetFunction()) {
                throw new IllegalStateException(
                    BuiltInRegistries.BLOCK.getKey(block) + " has states that disagree on the offset function"
                );
            }
        }

        var state = block.defaultBlockState();
        if (!state.hasOffsetFunction()) {
            return null;
        }

        // The offset type is only visible through the lambda it produced: XZ
        // never moves along Y, XYZ moves along Y for every seed nibble but 15.
        var movesVertically = false;
        for (int i = 0; i < OFFSET_TYPE_PROBES; i++) {
            if (state.getOffset(new BlockPos(i * 31, 0, i * -17)).y != 0.0) {
                movesVertically = true;
                break;
            }
        }

        var offsetJson = new JsonObject();
        offsetJson.addProperty("type", movesVertically ? "xyz" : "xz");
        offsetJson.addProperty("max_horizontal", (float) maxHorizontalOffset.invoke(block));
        offsetJson.addProperty("max_vertical", (float) maxVerticalOffset.invoke(block));

        var samplesJson = new JsonArray();
        for (var pos : OFFSET_SAMPLE_POSITIONS) {
            var offset = state.getOffset(pos);
            var sampleJson = new JsonObject();
            sampleJson.add("pos", ints(pos.getX(), pos.getY(), pos.getZ()));
            var offsetArray = new JsonArray();
            offsetArray.add(offset.x);
            offsetArray.add(offset.y);
            offsetArray.add(offset.z);
            sampleJson.add("offset", offsetArray);
            samplesJson.add(sampleJson);
        }
        offsetJson.add("samples", samplesJson);

        return offsetJson;
    }

    private static JsonArray ints(int... values) {
        var array = new JsonArray();
        for (var value : values) {
            array.add(value);
        }
        return array;
    }
}
