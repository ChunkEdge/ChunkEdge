package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import java.util.Optional;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.component.DataComponentMap;
import net.minecraft.core.component.DataComponents;
import net.minecraft.resources.RegistryOps;
import net.minecraft.server.MinecraftServer;
import net.minecraft.tags.DamageTypeTags;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.enchantment.Enchantable;

public class Items implements Main.Extractor {

    private final MinecraftServer server;

    public Items(MinecraftServer server) {
        this.server = server;
    }

    @Override
    public String fileName() {
        return "items.json";
    }

    @Override
    public JsonElement extract() throws Exception {
        var itemsJson = new JsonArray();

        for (var item : BuiltInRegistries.ITEM) {
            var itemJson = new JsonObject();

            itemJson.addProperty(
                "id",
                BuiltInRegistries.ITEM.getId(item)
            );
            itemJson.addProperty(
                "name",
                BuiltInRegistries.ITEM.getKey(item).getPath()
            );
            Item realItem = item;
            itemJson.addProperty(
                "translation_key",
                realItem.getDescriptionId()
            );
            itemJson.addProperty(
                "max_stack",
                realItem.getDefaultMaxStackSize()
            );
            itemJson.addProperty(
                "max_durability",
                realItem.getDefaultInstance().getMaxDamage()
            );
            itemJson.addProperty(
                "enchantability",
                Optional.ofNullable(
                    realItem.components().get(DataComponents.ENCHANTABLE)
                )
                    .map(Enchantable::value)
                    .orElse(0)
            );
            itemJson.addProperty(
                "fireproof",
                Optional.ofNullable(
                    realItem.components().get(DataComponents.DAMAGE_RESISTANT)
                )
                    .map(x ->
                        x
                            .types()
                            .unwrap()
                            .left()
                            .map(t -> t == DamageTypeTags.IS_FIRE)
                            .orElse(false)
                    )
                    .orElse(false)
            );

            itemJson.add(
                "components",
                DataComponentMap.CODEC.encodeStart(
                    RegistryOps.create(
                        JsonOps.INSTANCE,
                        server.registryAccess()
                    ),
                    realItem.components()
                ).getOrThrow()
            );

            itemsJson.add(itemJson);
        }
        return itemsJson;
    }
}
