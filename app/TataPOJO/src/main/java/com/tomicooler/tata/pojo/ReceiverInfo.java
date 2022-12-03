package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class ReceiverInfo {
    public enum Type {
        GCM,
        SMS_HUMAN,
        SMS_MACHINE,
        SERVICE
    }
    @Order(0) public Type type;
    @Order(1) public String phoneNumber;
}
