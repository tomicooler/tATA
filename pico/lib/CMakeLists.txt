add_library(tata_lib
    scope_exit.h
    utils.h
    executor.h
    location.h
    gpslocator.h
    gsmlocator.h
    locationservice.h
)
set_target_properties(tata_lib PROPERTIES LINKER_LANGUAGE CXX)

if(NOT BUILD_FOR_PICO)
    add_executable(tst_location
        tst_location.cpp)
    target_link_libraries(tst_location PRIVATE tata_lib)

    enable_testing()

    add_test(
      NAME tst_location
      COMMAND $<TARGET_FILE:tst_location>
    )
endif()

