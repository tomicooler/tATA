package com.tomicooler.tata.pojo;

import com.tomicooler.tata.Order;

public class Watcher {
    @Order(0) public Call call;
    @Order(1) public Refresh refresh;
    @Order(2) public Park park;
    @Order(3) public ReceiverInfo receiver;
    @Order(4) public Service service;
}
