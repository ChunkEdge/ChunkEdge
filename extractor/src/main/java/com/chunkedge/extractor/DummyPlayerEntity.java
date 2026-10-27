package com.chunkedge.extractor;

import com.mojang.authlib.GameProfile;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.GameType;

public class DummyPlayerEntity extends Player {

    public static final DummyPlayerEntity INSTANCE;

    static {
        INSTANCE = Main.magicallyInstantiate(DummyPlayerEntity.class);

        INSTANCE.defineSynchedData(new SynchedEntityData.Builder(INSTANCE));
    }

    public DummyPlayerEntity(ServerLevel world, GameProfile gameProfile) {
        super(world, gameProfile);
        this.setPos(0, 70, 0);
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
