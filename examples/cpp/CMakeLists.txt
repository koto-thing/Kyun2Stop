cmake_minimum_required(VERSION 3.20)
project(k2s_ffi_example LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(k2s_ffi_example main.cpp)

target_include_directories(k2s_ffi_example PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/../../include"
)

# Rust側の release 出力を参照
set(K2S_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release")

if (MSVC)
    target_link_libraries(k2s_ffi_example PRIVATE "${K2S_LIB_DIR}/Kyun2Stop.dll.lib")

    add_custom_command(TARGET k2s_ffi_example POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${K2S_LIB_DIR}/Kyun2Stop.dll"
            "$<TARGET_FILE_DIR:k2s_ffi_example>/Kyun2Stop.dll"
    )
else()
    target_link_directories(k2s_ffi_example PRIVATE "${K2S_LIB_DIR}")
    target_link_libraries(k2s_ffi_example PRIVATE Kyun2Stop)
endif()

