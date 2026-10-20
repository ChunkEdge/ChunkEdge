package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.core.registries.BuiltInRegistries;

public class Sounds implements Main.Extractor {

    public Sounds() {}

    @Override
    public String fileName() {
        return "sounds.json";
    }

    @Override
    public JsonElement extract() throws Exception {
        var itemsJson = new JsonArray();

        for (var sound : BuiltInRegistries.SOUND_EVENT) {
            var itemJson = new JsonObject();
            itemJson.addProperty(
                "id",
                BuiltInRegistries.SOUND_EVENT.getId(sound)
            );
            itemJson.addProperty(
                "name",
                BuiltInRegistries.SOUND_EVENT.getKey(sound).getPath()
            );
            itemsJson.add(itemJson);
        }

        return itemsJson;
    }
}
