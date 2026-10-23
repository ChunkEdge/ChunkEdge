package com.chunkedge.extractor.mixin;

import net.minecraft.world.item.StandingAndWallBlockItem;
import net.minecraft.world.level.block.Block;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(StandingAndWallBlockItem.class)
public interface ExposeWallBlock {
    @Accessor
    Block getWallBlock();
}
