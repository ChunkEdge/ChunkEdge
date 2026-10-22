package com.chunkedge.extractor;

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
import net.minecraft.world.flag.FeatureFlagSet;
import net.minecraft.world.flag.FeatureFlags;
import net.minecraft.world.level.storage.PrimaryLevelData;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import com.chunkedge.extractor.extractors.*;
import sun.reflect.ReflectionFactory;

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
            element.getAsJsonObject().entrySet().forEach(entry ->
                entries.put(entry.getKey(), entry.getValue())
            );
            entries.forEach((key, value) ->
                sorted.add(key, sortObjectKeys(value))
            );
            return sorted;
        }
        if (element.isJsonArray()) {
            var sorted = new JsonArray();
            element.getAsJsonArray().forEach(value ->
                sorted.add(sortObjectKeys(value))
            );
            return sorted;
        }
        return element;
    }

    /**
     * Magically creates an instance of a <i>concrete</i> class without calling its constructor.
     */
    public static <T> T magicallyInstantiate(Class<T> clazz) {
        var rf = ReflectionFactory.getReflectionFactory();
        try {
            var objCon = Object.class.getDeclaredConstructor();
            var con = rf.newConstructorForSerialization(clazz, objCon);
            return clazz.cast(con.newInstance());
        } catch (Throwable e) {
            throw new IllegalArgumentException(
                "Failed to magically instantiate " + clazz.getName(),
                e
            );
        }
    }

    @Override
    public void onInitialize() {
        LOGGER.info("Starting extractors...");

        var extractors = new Extractor[] {
            new Attributes(),
            new CommandArgumentTypes(),
            new Blocks(),
            new Effects(),
            new Packets(),
            new Sounds(),
            new TranslationKeys(),
        };

        Path outputDirectory;
        try {
            outputDirectory = Files.createDirectories(
                Paths.get("chunkedge_extractor_output")
            );
        } catch (IOException e) {
            LOGGER.info("Failed to create output directory.", e);
            return;
        }

        var gson = new GsonBuilder()
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
                LOGGER.error(
                    "Extractor for \"" + ext.fileName() + "\" failed.",
                    e
                );
            }
        }

        ServerLifecycleEvents.SERVER_STARTED.register(server -> {
            LOGGER.info("Server starting, Running startup extractors...");
            if (
                server.getWorldData() instanceof PrimaryLevelData levelData
            ) {
                levelData.setDataConfiguration(
                    levelData
                        .getDataConfiguration()
                        .expandFeatures(
                            FeatureFlagSet.of(
                                FeatureFlags.VANILLA,
                                FeatureFlags.MINECART_IMPROVEMENTS,
                                FeatureFlags.REDSTONE_EXPERIMENTS,
                                FeatureFlags.TRADE_REBALANCE
                            )
                        )
                );
            }
            // TODO: make `Codec` implement `Extractor`
            // TODO: the way to get Codex has changed, this is not working anymore
            var packetRegistryExtractor = new PacketRegistries(server);
            try {
                var out = outputDirectory.resolve(
                    packetRegistryExtractor.fileName()
                );
                try (var writer = Files.newBufferedWriter(out, StandardCharsets.UTF_8)) {
                    gson.toJson(sortObjectKeys(packetRegistryExtractor.extract()), writer);
                }

                LOGGER.info("Wrote " + out.toAbsolutePath());
            } catch (Exception e) {
                LOGGER.error(
                    "Extractor for \"" +
                    packetRegistryExtractor.fileName() +
                    "\" failed.",
                    e
                );
            }

            var startupExtractors = new Extractor[] {
                new Tags(server),
                new Paintings(server),
                new Enchants(server),
                new Entities(server),
                new Items(server),
                new Misc(server),
                new Recipe(server),
            };

            for (var ext : startupExtractors) {
                try {
                    var out = outputDirectory.resolve(ext.fileName());
                    try (var writer = Files.newBufferedWriter(out, StandardCharsets.UTF_8)) {
                        gson.toJson(sortObjectKeys(ext.extract()), writer);
                    }
                    LOGGER.info("Wrote " + out.toAbsolutePath());
                } catch (Exception e) {
                    LOGGER.error(
                        "Extractor for \"" + ext.fileName() + "\" failed.",
                        e
                    );
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
