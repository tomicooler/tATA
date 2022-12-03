package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class ParkLocation {

    public static final float MINIMUM_PARK_LOCATION_ACCURACY = 150.0f; // meters

    @Order(0) public Position position;
    @Order(1) public float accuracy;

    public static ParkLocation copyFromCarLocation(final CarLocation carLocation) {
        ParkLocation parkLocation = new ParkLocation();
        parkLocation.position = new Position(
                carLocation.position.latitude,
                carLocation.position.longitude);
        parkLocation.accuracy = Math.max(carLocation.accuracy, MINIMUM_PARK_LOCATION_ACCURACY);
        return parkLocation;
    }
}
