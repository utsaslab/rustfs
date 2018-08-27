#
#  > File Name:       Macros.cmake
#  > Author:          Zeyuan Hu
#  > Mail:            iamzeyuanhu@utexas.edu
#  > Created Time:    8/25/18
#  > Description:
#   
#    Commonly-used cmake macros & functions


# CMake version of spdk_lib_list_to_files defined in spdk.common.mk of SPDK repo
function(spdk_lib_list_to_files LIBS SPDK_LIB_LIST)
    # We convert a list of SPDK lib names to the corresponding file names
    set(ARG_NUM 1)
    while (ARG_NUM LESS ${ARGC})
        set(arg ${ARGV${ARG_NUM}})
        MESSAGE(STATUS "name: ${arg}")
        LIST(APPEND FileNames ${SPDK_DIR}/lib/libspdk_${arg}.a)
        math(EXPR ARG_NUM "${ARG_NUM}+1")
    endwhile()
    set(${LIBS} "${FileNames}" PARENT_SCOPE)
endfunction(spdk_lib_list_to_files)

function(dpdk_lib_list_to_files LIBS DPDK_LIB_LIST)
    # We convert a list of DPDK lib names to the corresponding file names
    set(ARG_NUM 1)
    while (ARG_NUM LESS ${ARGC})
        set(arg ${ARGV${ARG_NUM}})
        MESSAGE(STATUS "name: ${arg}")
        LIST(APPEND FileNames ${DPDK_DIR}/lib/librte_${arg}.a)
        math(EXPR ARG_NUM "${ARG_NUM}+1")
    endwhile()
    set(${LIBS} "${FileNames}" PARENT_SCOPE)
endfunction(dpdk_lib_list_to_files)