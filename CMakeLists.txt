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

# We enable the verbose output to help with debug building process
set( CMAKE_VERBOSE_MAKEFILE  on )

# This project can use C11, but will gracefully decay down to C89.
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED OFF)
set(CMAKE_C_EXTENSIONS OFF)

# Let CMake know where to search for the commonly-used CMake macros
set(CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/CMakeModules)

# SPDK_INSTALL_DIR should point to the install location of the SPDK library
set(SPDK_INSTALL_DIR $ENV{SPDK_INSTALL_DIR})
if (NOT SPDK_INSTALL_DIR)
    message(SEND_ERROR "SPDK_INSTALL_DIR not found into the environment")
else ()
    message(STATUS "SPDK_INSTALL_DIR = $ENV{SPDK_INSTALL_DIR}")
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
include_directories(${SPDK_INSTALL_DIR}/include)

add_subdirectory(examples/hello_nvme_bdev)