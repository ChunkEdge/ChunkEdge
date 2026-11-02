package com.chunkedge.extractor;

import java.lang.reflect.Field;

/** Reads private Minecraft metadata that has no public accessor. */
public final class MinecraftReflection {
    private MinecraftReflection() {}

    // Extraction requires private tracked-data declarations and protocol ID fields.
    // We use public Minecraft APIs wherever they exist but we need this exception.
    @SuppressWarnings("PMD.AvoidAccessibilityAlteration")
    public static Object readField(Field field, Object instance) throws IllegalAccessException {
        field.setAccessible(true);
        return field.get(instance);
    }
}
