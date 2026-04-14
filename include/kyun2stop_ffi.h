#ifndef KYUN2STOP_FFI_H
#define KYUN2STOP_FFI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef _WIN32
  #ifdef K2S_BUILD_DLL
    #define K2S_API __declspec(dllexport)
  #else
    #define K2S_API __declspec(dllimport)
  #endif
#else
  #define K2S_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct K2sConfig {
    float sample_rate;
    float max_seconds;
    uint32_t channels;
} K2sConfig;

typedef enum K2sCurve {
    K2S_CURVE_LINEAR = 0,
    K2S_CURVE_SMOOTH = 1,
    K2S_CURVE_SLOW_START = 2,
    K2S_CURVE_QUICK_CUT = 3,
} K2sCurve;

typedef enum K2sSyncBeat {
    K2S_SYNC_EIGHT = 0,
    K2S_SYNC_QUARTER = 1,
    K2S_SYNC_HALF = 2,
    K2S_SYNC_ONE_BAR = 3,
    K2S_SYNC_TWO_BARS = 4,
} K2sSyncBeat;

typedef struct K2sProcessParams {
    bool trigger;
    float stop_time_sec;
    float start_time_sec;
    K2sCurve curve;
    bool use_sync;
    K2sSyncBeat sync_beat;
    double bpm;
    bool enable_filter;
} K2sProcessParams;

typedef struct K2sOpaqueHandle K2sOpaqueHandle;

K2S_API K2sOpaqueHandle* k2s_create(K2sConfig config);
K2S_API void k2s_destroy(K2sOpaqueHandle* handle);
K2S_API bool k2s_reset(K2sOpaqueHandle* handle);
K2S_API bool k2s_process_interleaved_f32(
    K2sOpaqueHandle* handle,
    const float* input,
    float* output,
    size_t frames,
    K2sProcessParams params
);

#ifdef __cplusplus
}
#endif

#endif

