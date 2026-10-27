package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.core.registries.BuiltInRegistries;

public class ItemComponentTypes implements Main.Extractor {
    @Override
    public String fileName() {
        return "item_component_types.json";
    }

    @Override
    public JsonElement extract() {
        var json = new JsonObject();
        var registry = BuiltInRegistries.DATA_COMPONENT_TYPE;
        for (var type : registry) {
            json.addProperty(registry.getKey(type).getPath(), registry.getId(type));
        }
        return json;
    }
}
