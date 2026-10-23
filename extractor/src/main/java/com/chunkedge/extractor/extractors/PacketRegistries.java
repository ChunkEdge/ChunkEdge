package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.Registry;
import net.minecraft.resources.RegistryDataLoader;
import net.minecraft.server.MinecraftServer;

public class PacketRegistries implements Main.Extractor {

    private final MinecraftServer server;

    public PacketRegistries(MinecraftServer server) {
        this.server = server;
    }

    public String fileName() {
        return "registry_codec.json";
    }

    public static <T> JsonObject mapJson(
        RegistryDataLoader.RegistryData<T> registryData,
        MinecraftServer server
    ) {
        var ops = server.registryAccess().createSerializationContext(
            JsonOps.INSTANCE
        );
        Registry<T> registry = server
            .registryAccess()
            .lookupOrThrow(registryData.key());
        JsonObject json = new JsonObject();
        registry
            .listElements()
            .forEach(entry -> {
                json.add(
                    entry.key().identifier().toString(),
                    registryData
                        .elementCodec()
                        .encodeStart(ops, entry.value())
                        .resultOrPartial(e ->
                            Main.LOGGER.error("Cannot encode json: {}", e)
                        )
                        .orElseThrow()
                );
            });
        return json;
    }

    public JsonElement extract() {
        JsonObject json = new JsonObject();
        for (var entry : RegistryDataLoader.SYNCHRONIZED_REGISTRIES) {
            json.add(
                entry.key().identifier().toString(),
                mapJson(entry, server)
            );
        }
        return json;
    }
}
