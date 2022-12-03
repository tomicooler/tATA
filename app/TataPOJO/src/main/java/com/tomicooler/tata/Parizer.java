package com.tomicooler.tata;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.Comparator;

class Parizer {

    private static final String DELIMITER = " ";
    private static final String SPACE = "_";
    private static final String NULL = "*";
    private static final String TRUE = "t";
    private static final String FALSE = "f";
    private static final String EMPTY = ";";
    private static final long FLOAT_PRECISION = 1000000L;
    private static final long DOUBLE_PRECISION = 10000000000L;

    public static String serialize(Object obj) {
        StringBuilder builder = new StringBuilder();
        try {
            serialize(obj, builder);
        } catch (IllegalAccessException e) {
            return null;
        }
        return builder.toString();
    }

    public static <T> T deserialize(String string, Class<T> clazz) {
        T obj;
        try {
            final String[] variables = string.split(DELIMITER);
            obj = clazz.newInstance();
            int index = 0;
            index = deserialize(obj, variables, index);
            if (index != variables.length) {
                obj = null;
            }
        } catch (Exception e) {
            obj = null;
        }
        return obj;
    }

    private static void serialize(Object obj, StringBuilder string) throws IllegalAccessException {
        if (obj == null) {
            return;
        }

        for (Field field : sortFields(obj.getClass().getFields())) {
            if (Modifier.isStatic(field.getModifiers()) || Modifier.isFinal(field.getModifiers())) {
                continue;
            }

            final Object value = field.get(obj);
            if (value == null) {
                append(string, NULL);
            } else if (!field.getType().isPrimitive() && (field.getType() != String.class) && !field.getType().isEnum()) {
                serialize(value, string);
            } else {
                String valueString;

                if (value instanceof Float) {
                    valueString = Long.toString((long) (((Float) value) * FLOAT_PRECISION), Character.MAX_RADIX);
                } else if (value instanceof Double) {
                    valueString = Long.toString((long) (((Double) value) * DOUBLE_PRECISION), Character.MAX_RADIX);
                } else if (value instanceof Integer) {
                    valueString = Integer.toString((Integer) value, Character.MAX_RADIX);
                } else if (value instanceof Long) {
                    valueString = Long.toString((Long) value, Character.MAX_RADIX);
                } else if (value instanceof Boolean) {
                    valueString = (Boolean) value ? TRUE : FALSE;
                } else if (value instanceof Enum) {
                    valueString = Integer.toString(((Enum) value).ordinal(), Character.MAX_RADIX);
                } else {
                    valueString = escape(value.toString());
                }

                append(string, valueString);
            }
        }
    }

    private static int deserialize(Object obj, String[] variables, int i) throws Exception {
        for (Field field : sortFields(obj.getClass().getFields())) {
            if (Modifier.isStatic(field.getModifiers()) || Modifier.isFinal(field.getModifiers())) {
                continue;
            }

            String variable = variables[i];

            if (variable.equals(NULL)) {
                ++i;
            } else if (!field.getType().isPrimitive() && (field.getType() != String.class) && !field.getType().isEnum()) {
                Object value = field.getType().newInstance();
                i = deserialize(value, variables, i);
                field.set(obj, value);
            } else {
                Object value = null;

                if (field.getType() == String.class) {
                    value = unescape(variable);
                } else {
                    if (field.getType().getSimpleName().toLowerCase().contains("float")) {
                        value = (float) Long.parseLong(variable, Character.MAX_RADIX) / (float) FLOAT_PRECISION;
                    } else if (field.getType().getSimpleName().toLowerCase().contains("double")) {
                        value = (double) Long.parseLong(variable, Character.MAX_RADIX) / (double) DOUBLE_PRECISION;
                    } else if (field.getType().getSimpleName().toLowerCase().contains("int")) {
                        value = Integer.parseInt(variable, Character.MAX_RADIX);
                    } else if (field.getType().getSimpleName().toLowerCase().contains("long")) {
                        value = Long.parseLong(variable, Character.MAX_RADIX);
                    } else if (field.getType().getSimpleName().toLowerCase().contains("boolean")) {
                        value = variable.equals("t");
                    } else if (field.getType().isEnum()) {
                        int ordinal = Integer.parseInt(variable, Character.MAX_RADIX);
                        for (Object o : field.getType().getEnumConstants()) {
                            Enum e = (Enum) o;
                            if (e.ordinal() == ordinal) {
                                value = e;
                                break;
                            }
                        }
                        if (value == null) {
                            throw new Exception("Could not parse enum, ordinal not found: " + ordinal);
                        }
                    }
                }

                field.set(obj, value);
                ++i;
            }
        }

        return i;
    }

    private static Field[] sortFields(Field[] fields) {
        Arrays.sort(fields, new Comparator<Field>() {
            @Override
            public int compare(Field o1, Field o2) {
                Order or1 = o1.getAnnotation(Order.class);
                Order or2 = o2.getAnnotation(Order.class);
                // nulls last
                if (or1 != null && or2 != null) {
                    return or1.value() - or2.value();
                } else if (or1 != null) {
                    return -1;
                } else if (or2 != null) {
                    return 1;
                }
                return o1.getName().compareTo(o2.getName());
            }
        });
        return fields;
    }

    private static void append(StringBuilder builder, String string) {
        if (builder.length() > 0) {
            builder.append(DELIMITER);
        }
        builder.append(string);
    }

    private static String escape(String string) {
        if (string.isEmpty()) {
            string = EMPTY;
        } else {
            string = string.replace(SPACE, SPACE + SPACE);
            string = string.replace(DELIMITER + DELIMITER, SPACE + SPACE + SPACE + SPACE);
            string = string.replace(DELIMITER, SPACE);
            string = string.replace(NULL, NULL + NULL);
            string = string.replace(EMPTY , EMPTY + EMPTY);
        }
        return string;
    }

    private static String unescape(String string) {
        if (string.equals(EMPTY)) {
            string = "";
        } else {
            string = string.replace(SPACE + SPACE, SPACE);
            string = string.replace(SPACE, DELIMITER);
            string = string.replace(NULL + NULL, NULL);
            string = string.replace(EMPTY + EMPTY, EMPTY);
        }
        return string;
    }
}
