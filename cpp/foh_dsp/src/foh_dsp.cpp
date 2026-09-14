#include "foh_dsp.h"

#include <cmath>
#include <cstddef>
#include <cstdint>

int foh_dsp_abi(void) { return FOH_DSP_ABI; }

int foh_levels(const float *samples, size_t n, FohLevel *out) {
    if (!samples || !out || n == 0) {
        return -1;
    }
    float peak = 0.0f;
    double sum_sq = 0.0;
    double dc = 0.0;
    for (size_t i = 0; i < n; ++i) {
        const float a = std::fabs(samples[i]);
        if (a > peak) {
            peak = a;
        }
        sum_sq += static_cast<double>(samples[i]) * static_cast<double>(samples[i]);
        dc += samples[i];
    }
    out->peak = peak;
    out->rms = static_cast<float>(std::sqrt(sum_sq / static_cast<double>(n)));
    out->dc = static_cast<float>(dc / static_cast<double>(n));
    return 0;
}

int foh_peak_eq(float *samples, size_t n, uint32_t sr, float freq_hz, float q, float gain_db) {
    if (!samples || n == 0 || sr == 0 || freq_hz <= 0.0f || q <= 0.0f) {
        return -1;
    }
    const float A = std::pow(10.0f, gain_db / 40.0f);
    const float w = 2.0f * static_cast<float>(M_PI) * freq_hz / static_cast<float>(sr);
    const float cosw = std::cos(w);
    const float sinw = std::sin(w);
    const float alpha = sinw / (2.0f * q);
    const float b0 = 1.0f + alpha * A;
    const float b1 = -2.0f * cosw;
    const float b2 = 1.0f - alpha * A;
    const float a0 = 1.0f + alpha / A;
    const float a1 = -2.0f * cosw;
    const float a2 = 1.0f - alpha / A;
    const float B0 = b0 / a0;
    const float B1 = b1 / a0;
    const float B2 = b2 / a0;
    const float A1 = a1 / a0;
    const float A2 = a2 / a0;
    float z1 = 0.0f;
    float z2 = 0.0f;
    for (size_t i = 0; i < n; ++i) {
        const float x = samples[i];
        const float y = B0 * x + z1;
        z1 = B1 * x - A1 * y + z2;
        z2 = B2 * x - A2 * y;
        samples[i] = y;
    }
    return 0;
}
