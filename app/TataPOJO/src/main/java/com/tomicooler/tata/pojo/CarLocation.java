package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class CarLocation {
    @Order(0) public Position position;
    @Order(1) public float accuracy;
    @Order(2) public float battery;
    @Order(3) public long timestamp;
}
