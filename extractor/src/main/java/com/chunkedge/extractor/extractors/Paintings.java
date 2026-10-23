package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.RegistryOps;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.entity.decoration.painting.PaintingVariant;

public class Paintings implements Main.Extractor {

    private final MinecraftServer server;

    public Paintings(MinecraftServer server) {
        this.server = server;
    }

    @Override
    public String fileName() {
        return "paintings.json";
    }

    @Override
    public JsonElement extract() throws Exception {
        var paintingRegistry = server
            .registryAccess()
            .lookupOrThrow(Registries.PAINTING_VARIANT);

        var codec = PaintingVariant.DIRECT_CODEC;

        JsonObject json = new JsonObject();
        paintingRegistry
            .listElements()
            .forEach(entry -> {
                json.add(
                    entry.key().identifier().toString(),
                    codec
                        .encodeStart(
                            RegistryOps.create(
                                JsonOps.INSTANCE,
                                server.registryAccess()
                            ),
                            entry.value()
                        )
                        .getOrThrow()
                );
            });

        return json;
    }
}
