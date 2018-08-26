#
#  > File Name:       CMakeLists.txt
#  > Author:          Zeyuan Hu
#  > Mail:            iamzeyuanhu@utexas.edu
#  > Created Time:    8/25/18
#  > Description:
#   
#    This file is used to compile .c files shipped inside the project.

cmake_minimum_required(VERSION 2.8)
project(rustfs)

set(CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/CMakeModules)

# SPDK_DIR should point to the install location of the SPDK library
set(SPDK_DIR $ENV{SPDK_DIR})
if (NOT SPDK_DIR)
    message(SEND_ERROR "SPDK_DIR not found into the environment")
else ()
    message(STATUS "SPDK_DIR = $ENV{SPDK_DIR}")
endif ()

# DPDK_DIR should point to the install location of the DPDK library
# Note: please use the mirror of DPDK library shipped with SPDK library
# (DPDK library is maintained as a submodule of SPDK library. Thus, the best way to access the DPDK library is by
# following the submodule link from SPDK library git repo: https://github.com/spdk/spdk)
# Build the SPDK library will automatically build the DPDK library submodule. Then, you can point
# DPDK_DIR to the build directory (i.e. spdk/dpdk/build)
set(DPDK_DIR $ENV{DPDK_DIR})
if (NOT DPDK_DIR)
    message(SEND_ERROR "DPDK_DIR not found into the environment")
else ()
    message(STATUS "DPDK_DIR = $ENV{DPDK_DIR}")
endif ()

# Setup the basic C Compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} \
        -Wall \
        -Wextra \
        -Wno-unused-parameter \
        -Wno-missing-field-initializers \
        -Wmissing-declarations \
        -fno-strict-aliasing \
        -Wformat \
        -Wformat-security \
    ")

# include the SPDK API headers
include_directories(${SPDK_DIR}/include)

add_subdirectory(examples)