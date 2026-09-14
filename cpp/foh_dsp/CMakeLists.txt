cmake_minimum_required(VERSION 3.16)
project(foh_dsp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(foh_dsp STATIC
    src/foh_dsp.cpp
)
target_include_directories(foh_dsp PUBLIC include)

add_executable(foh_dsp_selftest src/selftest.cpp)
target_link_libraries(foh_dsp_selftest PRIVATE foh_dsp)
target_include_directories(foh_dsp_selftest PRIVATE include)
