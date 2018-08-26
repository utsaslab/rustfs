#
#  > File Name:       CMakeLists.txt
#  > Author:          Zeyuan Hu
#  > Mail:            iamzeyuanhu@utexas.edu
#  > Created Time:    8/25/18
#  > Description:
#   
#    Compile .c programs inside examples directory

# link to the SPDK libraries
link_directories(${SPDK_DIR}/lib)

# link to the DPDK libraries
link_directories(${DPDK_DIR}/lib)

# import Macros.cmake
include(Macros)

### TARGET: hello_nvme_bdev
add_executable(hello_nvme_bdev
        hello_nvme_bdev/hello_nvme_bdev.c
        )

spdk_lib_list_to_files(SG1
        copy_ioat
        ioat
        )

spdk_lib_list_to_files(SG2
        vbdev_lvol
        blob
        blob_bdev
        lvol
        bdev_malloc
        bdev_null
        bdev_nvme
        nvme
        vbdev_passthru
        vbdev_error
        vbdev_gpt
        vbdev_split
        bdev_aio
        bdev_virtio
        virtio
        )

spdk_lib_list_to_files(SG3
        sock
        sock_posix
        )

spdk_lib_list_to_files(SG4
        event_bdev
        event_copy
        )

spdk_lib_list_to_files(SG5
        bdev
        copy
        event
        thread
        util
        conf
        trace
        log
        jsonrpc
        json
        rpc
        env_dpdk)

dpdk_lib_list_to_files(DG1
        eal
        mempool
        ring
        mempool_ring
        pci
        bus_pci
        kvargs)

# This link is adapted from $(LINK_C) value from spdk/examples/bdev/hello_world Makefile
target_link_libraries(hello_nvme_bdev
        -Wl,-z,relro,-z,now -Wl,-z,noexecstack -pthread
        -Wl,--whole-archive ${SG1} -Wl,--no-whole-archive
        -Wl,--whole-archive ${SG2} -Wl,--no-whole-archive
        -laio
        -Wl,--whole-archive ${SG3} -Wl,--no-whole-archive
        -Wl,--whole-archive ${SG4} -Wl,--no-whole-archive
        ${SG5}
        -Wl,--start-group -Wl,--whole-archive ${DG1} -Wl,--end-group
        -Wl,--no-whole-archive -lnuma -ldl -lrt -luuid -lcrypto
        )
