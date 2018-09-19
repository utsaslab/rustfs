#
#  > File Name:       CMakeLists.txt
#  > Author:          Zeyuan Hu
#  > Mail:            iamzeyuanhu@utexas.edu
#  > Created Time:    8/25/18
#  > Description:
#   
#    Compile .c programs inside examples directory

# link to the SPDK libraries
link_directories(${SPDK_INSTALL_DIR}/lib)

### TARGET: hello_nvme_bdev
add_executable(hello_nvme_bdev
        hello_nvme_bdev.c
        )

# This link is adapted from $(LINK_C) value from spdk/examples/bdev/hello_world Makefile
target_link_libraries(hello_nvme_bdev
        ${SPDK_INSTALL_DIR}/lib/libspdk.so
        )
