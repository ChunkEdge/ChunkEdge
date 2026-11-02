package com.chunkedge.extractor;

import com.mojang.authlib.GameProfile;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.GameType;

public class DummyPlayerEntity extends Player {

    private DummyPlayerEntity(ServerLevel world, GameProfile gameProfile) {
        super(world, gameProfile);
    }

    public static DummyPlayerEntity create(ServerLevel world, GameProfile gameProfile) {
        var player = new DummyPlayerEntity(world, gameProfile);
        player.setPos(0, 70, 0);
        return player;
    }

    @Override
    public GameType gameMode() {
        return GameType.SURVIVAL;
    }

    @Override
    public boolean isSpectator() {
        return false;
    }

    @Override
    public boolean isCreative() {
        return false;
    }
}
