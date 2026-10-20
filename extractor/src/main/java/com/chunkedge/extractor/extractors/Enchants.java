package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.RegistryOps;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.item.enchantment.Enchantment;

public class Enchants implements Main.Extractor {

    private final MinecraftServer server;

    public Enchants(MinecraftServer server) {
        this.server = server;
    }

    @Override
    public String fileName() {
        return "enchants.json";
    }

    @Override
    public JsonElement extract() {
        var enchantsJson = new JsonObject();

        var lookup = server
            .registryAccess()
            .lookupOrThrow(Registries.ENCHANTMENT);

        for (var enchant : lookup.listElements().toList()) {
            enchantsJson.add(
                enchant.key().identifier().toString(),
                Enchantment.DIRECT_CODEC.encodeStart(
                    RegistryOps.create(
                        JsonOps.INSTANCE,
                        server.registryAccess()
                    ),
                    enchant.value()
                ).getOrThrow()
            );
        }

        return enchantsJson;
    }
}
