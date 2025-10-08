include(FetchContent)

FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.6.0
)
FetchContent_MakeAvailable(Catch2)

add_executable(test_vec3 test_vec3.cpp)
target_link_libraries(test_vec3 PRIVATE Catch2::Catch2WithMain molomni_c)
target_compile_features(test_vec3 PRIVATE cxx_std_17)
target_include_directories(test_vec3 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/..)

add_test(NAME vec3 COMMAND test_vec3)
