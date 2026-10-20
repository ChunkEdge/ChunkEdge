package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.lang.reflect.Field;
import java.util.Map;
import net.minecraft.locale.Language;

public class TranslationKeys implements Main.Extractor {

    @Override
    public String fileName() {
        return "translation_keys.json";
    }

    @Override
    public JsonElement extract() {
        JsonArray translationsJson = new JsonArray();

        Map<String, String> translations = extractTranslations();
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

    @SuppressWarnings("unchecked")
    private static Map<String, String> extractTranslations() {
        Language language = Language.getInstance();

        Class<? extends Language> anonymousClass = language.getClass();
        for (Field field : anonymousClass.getDeclaredFields()) {
            try {
                field.setAccessible(true);
                Object fieldValue = field.get(language);
                if (fieldValue instanceof Map<?, ?>) {
                    return (Map<String, String>) fieldValue;
                }
            } catch (IllegalAccessException | SecurityException e) {
                throw new RuntimeException(
                    "Failed reflection on field '" +
                    field +
                    "' on class '" +
                    anonymousClass +
                    "'",
                    e
                );
            }
        }

        throw new RuntimeException(
            "Did not find anonymous map under 'net.minecraft.locale.Language.loadDefault()'"
        );
    }
}
