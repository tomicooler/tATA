package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class Position {
    @Order(0) public double latitude;
    @Order(1) public double longitude;

    public Position() {
    }

    public Position(double latitude, double longitude) {
        this.latitude = latitude;
        this.longitude = longitude;
    }
}
