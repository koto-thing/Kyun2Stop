#pragma once

#include <stdexcept>
#include <utility>

#include "../../include/kyun2stop_ffi.h"

class K2sEngine {
public:
    explicit K2sEngine(const K2sConfig& config) : handle_(k2s_create(config)) {
        if (handle_ == nullptr) {
            throw std::runtime_error("k2s_create failed");
        }
    }

    ~K2sEngine() {
        if (handle_ != nullptr) {
            k2s_destroy(handle_);
        }
    }

    K2sEngine(const K2sEngine&) = delete;
    K2sEngine& operator=(const K2sEngine&) = delete;

    K2sEngine(K2sEngine&& other) noexcept : handle_(other.handle_) {
        other.handle_ = nullptr;
    }

    K2sEngine& operator=(K2sEngine&& other) noexcept {
        if (this != &other) {
            if (handle_ != nullptr) {
                k2s_destroy(handle_);
            }
            handle_ = other.handle_;
            other.handle_ = nullptr;
        }
        return *this;
    }

    bool reset() {
        return k2s_reset(handle_);
    }

    bool processInterleavedF32(const float* input, float* output, size_t frames, const K2sProcessParams& params) {
        return k2s_process_interleaved_f32(handle_, input, output, frames, params);
    }

private:
    K2sOpaqueHandle* handle_ = nullptr;
};

