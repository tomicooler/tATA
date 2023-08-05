cmake_minimum_required(VERSION 3.25)

option(BUILD_FOR_PICO "Build for Raspberry Pico" OFF)

if(BUILD_FOR_PICO)
    include(pico_sdk_import.cmake)
    include(pico_extras_import.cmake)
endif()

project(tata C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(BUILD_FOR_PICO)
    if (PICO_SDK_VERSION_STRING VERSION_LESS "1.3.0")
        message(FATAL_ERROR "Raspberry Pi Pico SDK version 1.3.0 (or later) required. Your version is ${PICO_SDK_VERSION_STRING}")
    endif()

    pico_sdk_init()
    add_compile_definitions(PICO_UART_ENABLE_CRLF_SUPPORT=0)
endif()


add_compile_options(
	-Wall
        -Wno-format               # int != int32_t as far as the compiler is concerned because gcc has int32_t as long int
        -Wno-unused-function      # we have some for the docs that aren't called
        -Wno-maybe-uninitialized
)

add_subdirectory(poco)
add_subdirectory(lib)

if(BUILD_FOR_PICO)
    add_subdirectory(app)
else()
    enable_testing()
endif()

