#include <catch2/catch_test_macros.hpp>
#include <cstdint>
#include <vector>

extern "C" {
#include "molcore.h"
}

TEST_CASE("vec3 from/to ptr f32", "[vec3]") {
    float data[3] = {1.f, 2.f, 3.f};
    mc_vec3f32 v{};
    REQUIRE(mc_vec3f32_from_ptr(data, &v) == 0);
    REQUIRE(v.x == 1.f);
    REQUIRE(v.y == 2.f);
    REQUIRE(v.z == 3.f);

    float out[3] = {0.f, 0.f, 0.f};
    REQUIRE(mc_vec3f32_to_ptr(&v, out, 3) == 0);
    CHECK(out[0] == 1.f);
    CHECK(out[1] == 2.f);
    CHECK(out[2] == 3.f);

    REQUIRE(mc_vec3f32_scale_inplace(out, 2.f, 3) == 0);
    CHECK(out[0] == 2.f);
    CHECK(out[1] == 4.f);
    CHECK(out[2] == 6.f);

    float addend[3] = {1.f, 1.f, 1.f};
    REQUIRE(mc_vec3f32_add_inplace(out, addend, 3) == 0);
    CHECK(out[0] == 3.f);
    CHECK(out[1] == 5.f);
    CHECK(out[2] == 7.f);
}

TEST_CASE("vec3 from/to ptr f64", "[vec3]") {
    double data[3] = {1.0, 2.0, 3.0};
    mc_vec3f64 v{};
    REQUIRE(mc_vec3f64_from_ptr(data, &v) == 0);
    REQUIRE(v.x == 1.0);
    REQUIRE(v.y == 2.0);
    REQUIRE(v.z == 3.0);

    double out[3] = {0.0, 0.0, 0.0};
    REQUIRE(mc_vec3f64_to_ptr(&v, out, 3) == 0);
    CHECK(out[0] == 1.0);
    CHECK(out[1] == 2.0);
    CHECK(out[2] == 3.0);

    REQUIRE(mc_vec3f64_scale_inplace(out, 0.5, 3) == 0);
    CHECK(out[0] == 0.5);
    CHECK(out[1] == 1.0);
    CHECK(out[2] == 1.5);

    double addend[3] = {0.5, 0.5, 0.5};
    REQUIRE(mc_vec3f64_add_inplace(out, addend, 3) == 0);
    CHECK(out[0] == 1.0);
    CHECK(out[1] == 1.5);
    CHECK(out[2] == 2.0);
}
