#include <cmath>
#include <cstdint>
#include <iostream>
#include <vector>

#include "k2s_raii.hpp"

int main() {
    constexpr uint32_t channels = 2;
    constexpr size_t frames = 256;

    K2sConfig config{};
    config.sample_rate = 48000.0f;
    config.max_seconds = 3.0f;
    config.channels = channels;

    try {
        K2sEngine engine(config);

        std::vector<float> input(frames * channels, 0.0f);
        std::vector<float> output(frames * channels, 0.0f);

        // 1 kHzの簡易テストトーン
        for (size_t i = 0; i < frames; ++i) {
            float s = std::sin(2.0f * 3.14159265f * 1000.0f * static_cast<float>(i) / 48000.0f);
            input[i * channels + 0] = s;
            input[i * channels + 1] = s;
        }

        K2sProcessParams params{};
        params.trigger = true;
        params.stop_time_sec = 0.5f;
        params.start_time_sec = 0.5f;
        params.curve = K2S_CURVE_SMOOTH;
        params.use_sync = false;
        params.sync_beat = K2S_SYNC_QUARTER;
        params.bpm = 120.0;
        params.enable_filter = true;

        if (!engine.processInterleavedF32(input.data(), output.data(), frames, params)) {
            std::cerr << "k2s_process_interleaved_f32 failed" << std::endl;
            return 1;
        }

        std::cout << "Processed first sample L/R: "
                  << output[0] << ", " << output[1] << std::endl;

        return 0;
    } catch (const std::exception& e) {
        std::cerr << e.what() << std::endl;
        return 1;
    }
}

