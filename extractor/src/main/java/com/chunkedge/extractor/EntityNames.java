package com.chunkedge.extractor;

/** Mechanical names for entity marker components and their tracked data. */
public final class EntityNames {

    private EntityNames() {}

    /** The suffix distinguishes Minecraft marker components from Bevy entity handles. */
    public static String className(Class<?> type) {
        var name = type.getSimpleName();
        return name.endsWith("Entity") ? name : name + "Entity";
    }

    /** Remove accessor bookkeeping, retaining the official semantic name. */
    public static String fieldName(String name) {
        return name
            .replaceFirst("^(data_)?(id_)?", "")
            .replaceFirst("_id$", "");
    }
}
