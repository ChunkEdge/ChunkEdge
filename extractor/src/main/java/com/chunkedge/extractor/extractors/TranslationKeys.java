package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.Map;
import java.util.TreeMap;
import net.minecraft.locale.Language;

public class TranslationKeys implements Main.Extractor {

    @Override
    public String fileName() {
        return "translation_keys.json";
    }

    @Override
    public JsonElement extract() {
        JsonArray translationsJson = new JsonArray();

        Map<String, String> translations = new TreeMap<>(extractTranslations());
        for (var translation : translations.entrySet()) {
            String translationKey = translation.getKey();
            String translationValue = translation.getValue();

            var translationJson = new JsonObject();
            translationJson.addProperty("key", translationKey);
            translationJson.addProperty(
                "english_translation",
                translationValue
            );

            translationsJson.add(translationJson);
        }

        return translationsJson;
    }

    private static Map<String, String> extractTranslations() {
        // Read vanilla directly: Fabric 26.3 injects translation keys whose
        // normalized Rust constant names collide with each other.
        var translations = new TreeMap<String, String>();
        try (var stream = Language.class.getResourceAsStream("/assets/minecraft/lang/en_us.json")) {
            Language.loadFromJson(stream, translations::put);
        } catch (java.io.IOException e) {
            throw new RuntimeException("Failed to read vanilla translations", e);
        }
        net.minecraft.locale.DeprecatedTranslationsInfo.loadFromDefaultResource().applyToMap(translations);
        return translations;
    }
}
