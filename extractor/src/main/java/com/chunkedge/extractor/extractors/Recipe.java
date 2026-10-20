package com.chunkedge.extractor.extractors;

import com.chunkedge.extractor.Main;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.RegistryOps;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.item.crafting.RecipeManager;

public class Recipe implements Main.Extractor {

    private final MinecraftServer server;
    private final RecipeManager recipeManager;

    public Recipe(MinecraftServer server) {
        this.server = server;
        this.recipeManager = server.getRecipeManager();
    }

    @Override
    public String fileName() {
        return "recipes.json";
    }

    @Override
    public JsonElement extract() throws Exception {
        var ops = RegistryOps.create(
            JsonOps.INSTANCE,
            server.registryAccess()
        );
        JsonObject json = new JsonObject();

        JsonObject recipesJson = new JsonObject();
        recipeManager
            .getRecipes()
            .forEach(holder -> {
                recipesJson.add(
                    holder.id().identifier().getPath(),
                    net.minecraft.world.item.crafting.Recipe.CODEC.encodeStart(ops, holder.value()).getOrThrow()
                );
            });

        JsonObject displaysJson = new JsonObject();
        var displays = server
            .registryAccess()
            .lookupOrThrow(Registries.RECIPE_DISPLAY);
        displays
            .stream()
            .forEach(display -> {
                displaysJson.addProperty(
                    displays
                        .byNameCodec()
                        .encodeStart(ops, display)
                        .getOrThrow()
                        .getAsString(),
                    displays.getId(display)
                );
            });

        JsonObject bookCategoryJson = new JsonObject();
        var bookCategory = server
            .registryAccess()
            .lookupOrThrow(Registries.RECIPE_BOOK_CATEGORY);

        bookCategory
            .listElements()
            .forEach(entry -> {
                bookCategoryJson.addProperty(
                    bookCategory
                        .byNameCodec()
                        .encodeStart(ops, entry.value())
                        .getOrThrow()
                        .getAsString(),
                    bookCategory.getId(entry.value())
                );
            });

        json.add("recipes", recipesJson);
        json.add("displays", displaysJson);
        json.add("book_categories", bookCategoryJson);

        return json;
    }
}
