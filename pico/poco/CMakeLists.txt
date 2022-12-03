add_library(poco
    common.h
    protector.h
    watcher.h
    parizer.h
    machine.h
    machine.cpp
    human.h
    human.cpp
    poco.h
    debug.h
)
set_target_properties(poco PROPERTIES LINKER_LANGUAGE CXX)

if(NOT BUILD_FOR_PICO)
    add_executable(tst_poco
        tst_poco.cpp)
    target_link_libraries(tst_poco PRIVATE poco)

    enable_testing()

    add_test(
      NAME tst_poco
      COMMAND $<TARGET_FILE:tst_poco>
    )
endif()

