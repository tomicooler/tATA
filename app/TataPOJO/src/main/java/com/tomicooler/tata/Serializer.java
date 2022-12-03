package com.tomicooler.tata;

public class Serializer {
    public static String write(Object object) {
        return Parizer.serialize(object);
    }

    public static <T> T read(String string, Class<T> clazz) {
        return Parizer.deserialize(string, clazz);
    }
}
