package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class Protector {
    @Order(0) public CarLocation carLocation;
    @Order(1) public ParkLocation parkLocation;
    @Order(2) public Status status;
    @Order(3) public Service service;
}
