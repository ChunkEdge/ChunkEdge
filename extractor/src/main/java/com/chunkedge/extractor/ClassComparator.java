package com.chunkedge.extractor;

import java.io.Serializable;
import java.util.Comparator;

/**
 * Compare Class objects by their fully qualified names lexicographically.
 */
public class ClassComparator implements Comparator<Class<?>>, Serializable {

    public ClassComparator() {}

    @Override
    public int compare(Class<?> c1, Class<?> c2) {
        return c1.getName().compareTo(c2.getName());
    }

    @Override
    public boolean equals(Object comp) {
        return comp instanceof ClassComparator;
    }
}
