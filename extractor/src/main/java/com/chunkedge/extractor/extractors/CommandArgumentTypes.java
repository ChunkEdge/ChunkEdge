package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.core.registries.BuiltInRegistries;

public class CommandArgumentTypes implements Main.Extractor {

    @Override
    public String fileName() {
        return "command_argument_types.json";
    }

    @Override
    public JsonElement extract() {
        var json = new JsonObject();
        var registry = BuiltInRegistries.COMMAND_ARGUMENT_TYPE;
        for (var type : registry) {
            json.addProperty(registry.getKey(type).toString(), registry.getId(type));
        }
        return json;
    }
}
