package com.tomicooler.tata;

import com.tomicooler.tata.Order;
import com.tomicooler.tata.Parizer;

import static org.junit.Assert.assertEquals;
import org.junit.Test;

public class ParizerTest {

    static class A {
        enum E {
            ABC,
            DEF,
            GHI
        }

        static class Single {
            public float a;
        }

        static class B {
            static class C {
                public static final String SKIP_STATIC_MEMBERS = "aa";
                @Order(0)
                public float a;
                @Order(1)
                public double b;
            }

            @Order(0)
            public int a;
            @Order(1)
            public long b;
            @Order(2)
            public C c;
        }

        @Order(0)
        public Single a; // Currently Object without a public field is not supported
        @Order(1)
        public B b;
        @Order(2)
        public String c;
        @Order(3)
        public String d;
        @Order(4)
        public String e;
        @Order(5)
        public String f;
        @Order(6)
        public String g;
        @Order(7)
        public String h;
        @Order(8)
        public String i;
        @Order(9)
        public E j;
    }

    private static class ET {
        public A.E e;
    }

    static class OrderTestClassA {
        @Order(0)
        public int a;
        @Order(2)
        public int b;
        @Order(1)
        public int c;
    }

    static class OrderTestClassB {
        public int a;
        public int b;
        public int c;
    }

    @Test
    public void testParizer() throws Exception {
        A a = new A();
        a.a = null;
        a.b = new A.B();
        a.b.a = 123;
        a.b.b = 123456789L;
        a.b.c = new A.B.C();
        a.b.c.a = 123.123123f;
        a.b.c.b = 123.1231231231d;
        a.c = "string";
        a.d = "*";
        a.e = " a b c d e   f g h ";
        a.f = null;
        a.g = "t";
        a.h = "f";
        a.i = "";
        a.j = A.E.DEF;

        String serializedA = Parizer.serialize(a);
        A deserializedA = Parizer.deserialize(serializedA, A.class);

        assertEquals(serializedA, Parizer.serialize(deserializedA));
    }

    @Test
    public void testSerializeErrorCases() throws Exception {
        assertEquals("", Parizer.serialize(null));
    }

    @Test
    public void testDeserializeErrorCases() throws Exception {
        assertEquals(null, Parizer.deserialize("", A.class));
        assertEquals(null, Parizer.deserialize("1 2 3 2", A.class)); // less
        assertEquals(null, Parizer.deserialize("1 2 1", A.B.C.class)); // more
        assertEquals(null, Parizer.deserialize("100", ET.class)); // invalid enum ordinal
    }

    @Test
    public void testOrder() throws Exception {
        OrderTestClassA a = new OrderTestClassA();
        a.a = 0;
        a.b = 2;
        a.c = 1;

        assertEquals("0 1 2", Parizer.serialize(a));

        OrderTestClassB b = new OrderTestClassB();
        b.a = 0;
        b.b = 2;
        b.c = 1;

        assertEquals("0 2 1", Parizer.serialize(b));
    }

    static class EmptyStringTest {
        @Order(0)
        public String a;
        @Order(1)
        public String b;
        @Order(2)
        public String c;
        @Order(3)
        public String d;
    }

    @Test
    public void testEmptyString() throws Exception {
        EmptyStringTest a = new EmptyStringTest();
        a.a = null;
        a.b = "";
        a.c = ";";
        a.d = ";;";

        String serializedA = Parizer.serialize(a);
        assertEquals("* ; ;; ;;;;", Parizer.serialize(a));

        EmptyStringTest deserializedA = Parizer.deserialize(serializedA, EmptyStringTest.class);
        assertEquals(serializedA, Parizer.serialize(deserializedA));
    }

    @Test
    public void testStaticFinalInMainClass() throws Exception {
        A.B.C a = new A.B.C();
        a.a = 123.123f;
        a.b = 123.123d;
        String serializedA = Parizer.serialize(a);
        A.B.C deserializedA = Parizer.deserialize(serializedA, A.B.C.class);

        assertEquals(serializedA, Parizer.serialize(deserializedA));
    }
}