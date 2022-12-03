add_executable(app
    sim868.h
    sim868.cpp
    main.cpp
)

target_link_libraries(app poco pico_stdlib hardware_spi hardware_adc hardware_interp hardware_sleep)

pico_enable_stdio_usb(app 1)
pico_enable_stdio_uart(app 0)

pico_add_extra_outputs(app)

