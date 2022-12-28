add_library(tata_lib
    scope_exit.h
    utils.h
    executor.h
    location.h
    gpslocator.h
    gsmlocator.h
    locationservice.h
    caller.h
    smssender.h
    network.h
)
set_target_properties(tata_lib PROPERTIES LINKER_LANGUAGE CXX)
target_link_libraries(tata_lib PRIVATE fmt::fmt)

if(NOT BUILD_FOR_PICO)
    enable_testing()

    add_executable(tst_location testutils.h tst_location.cpp)
    target_link_libraries(tst_location PRIVATE tata_lib)
    add_test(
      NAME tst_location
      COMMAND $<TARGET_FILE:tst_location>
    )

    add_executable(tst_phone testutils.h tst_phone.cpp)
    target_link_libraries(tst_phone PRIVATE tata_lib)

    add_test(
      NAME tst_phone
      COMMAND $<TARGET_FILE:tst_phone>
    )
endif()

