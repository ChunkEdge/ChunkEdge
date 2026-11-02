package com.chunkedge.extractor;

import com.chunkedge.extractor.extractors.*;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.TreeMap;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Main implements ModInitializer {

    public static final String MOD_ID = "chunkedge_extractor";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    // Codecs can build JSON objects from maps with unspecified iteration order.
    // Sort object keys recursively while retaining array order, to make
    // the extractor output deterministic.
    private static JsonElement sortObjectKeys(JsonElement element) {
        if (element.isJsonObject()) {
            var sorted = new JsonObject();
            var entries = new TreeMap<String, JsonElement>();
            element.getAsJsonObject()
                    .entrySet()
                    .forEach(entry -> entries.put(entry.getKey(), entry.getValue()));
            entries.forEach((key, value) -> sorted.add(key, sortObjectKeys(value)));
            return sorted;
        }
        if (element.isJsonArray()) {
            var sorted = new JsonArray();
            element.getAsJsonArray().forEach(value -> sorted.add(sortObjectKeys(value)));
            return sorted;
        }
        return element;
    }

    @Override
    public void onInitialize() {
        LOGGER.info("Starting extractors...");

        var extractors =
                new Extractor[] {
                    new Attributes(),
                    new CommandArgumentTypes(),
                    new ItemComponentTypes(),
                    new Effects(),
                    new Packets(),
                    new Sounds(),
                    new TranslationKeys(),
                };

        Path outputDirectory;
        try {
            outputDirectory = Files.createDirectories(Paths.get("chunkedge_extractor_output"));
        } catch (IOException e) {
            LOGGER.info("Failed to create output directory.", e);
            return;
        }

        var gson =
                new GsonBuilder()
                        .setPrettyPrinting()
                        .disableHtmlEscaping()
                        .serializeNulls()
                        .create();

        for (var ext : extractors) {
            try {
                var out = outputDirectory.resolve(ext.fileName());
                try (var writer = Files.newBufferedWriter(out, StandardCharsets.UTF_8)) {
                    gson.toJson(sortObjectKeys(ext.extract()), writer);
                }
                LOGGER.info("Wrote " + out.toAbsolutePath());
            } catch (Exception e) {
                LOGGER.error("Extractor for \"" + ext.fileName() + "\" failed.", e);
            }
        }

        ServerLifecycleEvents.SERVER_STARTED.register(
                server -> {
                    LOGGER.info("Server starting, Running startup extractors...");
                    // TODO: make `Codec` implement `Extractor`
                    // TODO: the way to get Codex has changed, this is not working anymore
                    var packetRegistryExtractor = new PacketRegistries(server);
                    try {
                        var out = outputDirectory.resolve(packetRegistryExtractor.fileName());
                        try (var writer = Files.newBufferedWriter(out, StandardCharsets.UTF_8)) {
                            gson.toJson(sortObjectKeys(packetRegistryExtractor.extract()), writer);
                        }

                        LOGGER.info("Wrote " + out.toAbsolutePath());
                    } catch (Exception e) {
                        LOGGER.error(
                                "Extractor for \""
                                        + packetRegistryExtractor.fileName()
                                        + "\" failed.",
                                e);
                    }

                    var startupExtractors =
                            new Extractor[] {
                                new Blocks(),
                                new Tags(server),
                                new Paintings(server),
                                new Enchants(server),
                                new Entities(server),
                                new EntityHitboxes(server),
                                new Items(server),
                                new Misc(server),
                                new Recipe(server),
                            };

                    for (var ext : startupExtractors) {
                        try {
                            var out = outputDirectory.resolve(ext.fileName());
                            try (var writer =
                                    Files.newBufferedWriter(out, StandardCharsets.UTF_8)) {
                                gson.toJson(sortObjectKeys(ext.extract()), writer);
                            }
                            LOGGER.info("Wrote " + out.toAbsolutePath());
                        } catch (Exception e) {
                            LOGGER.error("Extractor for \"" + ext.fileName() + "\" failed.", e);
                        }
                    }

                    LOGGER.info("Done.");
                    server.halt(false);
                });
    }

    public interface Extractor {
        String fileName();

        JsonElement extract() throws Exception;
    }

    public record Pair<T, U>(T left, U right) {}
}
