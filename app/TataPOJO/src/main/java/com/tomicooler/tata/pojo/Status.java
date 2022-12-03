package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class Status {
    public enum Type {
        PARKING_DETECTED,
        PARKING_UPDATED,
        CAR_THEFT_DETECTED
    }
    @Order(0) public Type status;
}
