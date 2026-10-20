package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.core.registries.BuiltInRegistries;

public class Effects implements Main.Extractor {

    public Effects() {}

    @Override
    public String fileName() {
        return "effects.json";
    }

    @Override
    public JsonElement extract() {
        var effectsJson = new JsonArray();

        for (var effect : BuiltInRegistries.MOB_EFFECT) {
            var effectJson = new JsonObject();

            effectJson.addProperty(
                "id",
                BuiltInRegistries.MOB_EFFECT.getId(effect)
            );
            effectJson.addProperty(
                "name",
                BuiltInRegistries.MOB_EFFECT.getKey(effect).getPath()
            );
            effectJson.addProperty(
                "translation_key",
                effect.getDescriptionId()
            );
            effectJson.addProperty("color", effect.getColor());
            effectJson.addProperty("instant", effect.isInstantaneous());
            effectJson.addProperty("category", effect.getCategory().name());

            var attributeModifiersJson = new JsonArray();

            effect.createModifiers(
                0,
                (attrHolder, modifier) -> {
                    var attributeModifierJson = new JsonObject();

                    attributeModifierJson.addProperty(
                        "attribute_name",
                        attrHolder
                            .value()
                            .getDescriptionId()
                            .replaceFirst("^attribute.name.", "")
                    );
                    attributeModifierJson.addProperty(
                        "operation",
                        modifier.operation().id()
                    );
                    attributeModifierJson.addProperty(
                        "base_value",
                        modifier.amount()
                    );

                    attributeModifiersJson.add(attributeModifierJson);
                }
            );

            if (!attributeModifiersJson.isEmpty()) {
                effectJson.add("attribute_modifiers", attributeModifiersJson);
            }

            effectsJson.add(effectJson);
        }

        return effectsJson;
    }
}
