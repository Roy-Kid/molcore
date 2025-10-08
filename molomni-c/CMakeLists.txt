cmake_minimum_required(VERSION 3.20)
project(molomni_c LANGUAGES C CXX)

# Path to a prebuilt Rust shared library (libmolomni_c.*). If not given, try to auto-detect
set(MOLOMNI_C_PREBUILT_LIB "" CACHE FILEPATH "Path to prebuilt libmolomni_c shared library")
option(MOLOMNI_C_ENABLE_INSTALL "Enable installation of headers and shared library" OFF)

# Compute platform-specific shared library extension
if(APPLE)
  set(_SO_EXT ".dylib")
elseif(WIN32)
  set(_SO_EXT ".dll")
else()
  set(_SO_EXT ".so")
endif()

# Try to auto-detect the library if not provided
if(NOT MOLOMNI_C_PREBUILT_LIB)
  set(_candidates)
  if(DEFINED ENV{CARGO_TARGET_DIR})
    list(APPEND _candidates $ENV{CARGO_TARGET_DIR}/debug/libmolomni_c${_SO_EXT})
    list(APPEND _candidates $ENV{CARGO_TARGET_DIR}/release/libmolomni_c${_SO_EXT})
  else()
    # Workspace default target directory is at repository root ../target
    list(APPEND _candidates ${CMAKE_CURRENT_LIST_DIR}/../target/debug/libmolomni_c${_SO_EXT})
    list(APPEND _candidates ${CMAKE_CURRENT_LIST_DIR}/../target/release/libmolomni_c${_SO_EXT})
  endif()
  foreach(_cand IN LISTS _candidates)
    if(EXISTS ${_cand})
      set(MOLOMNI_C_PREBUILT_LIB ${_cand})
      message(STATUS "Found prebuilt lib: ${MOLOMNI_C_PREBUILT_LIB}")
      break()
    endif()
  endforeach()
endif()

if(NOT MOLOMNI_C_PREBUILT_LIB)
  message(FATAL_ERROR "Could not find libmolomni_c. Please run 'cargo build -p molomni-c' first or set MOLOMNI_C_PREBUILT_LIB.")
endif()

# Interface library to propagate headers
add_library(molomni_c INTERFACE)
target_include_directories(molomni_c INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)

# Imported shared library target
add_library(molomni_c_shared SHARED IMPORTED GLOBAL)
set_target_properties(molomni_c_shared PROPERTIES IMPORTED_LOCATION ${MOLOMNI_C_PREBUILT_LIB})
target_link_libraries(molomni_c INTERFACE molomni_c_shared)

# Ensure the build tree can locate the shared lib at runtime
get_filename_component(_lib_dir ${MOLOMNI_C_PREBUILT_LIB} DIRECTORY)
set(CMAKE_BUILD_RPATH ${CMAKE_BUILD_RPATH};${_lib_dir})

enable_testing()
add_subdirectory(tests)

# Optional install logic (header + shared lib)
include(GNUInstallDirs)
if(MOLOMNI_C_ENABLE_INSTALL)
  install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/include/molcore.h DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
  if(WIN32)
    install(FILES ${MOLOMNI_C_PREBUILT_LIB} DESTINATION ${CMAKE_INSTALL_BINDIR})
  else()
    install(FILES ${MOLOMNI_C_PREBUILT_LIB} DESTINATION ${CMAKE_INSTALL_LIBDIR})
  endif()
endif()
