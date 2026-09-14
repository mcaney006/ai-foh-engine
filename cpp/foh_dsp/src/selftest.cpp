#include "foh_dsp.h"

#include <cmath>
#include <cstdio>
#include <vector>

int main() {
    if (foh_dsp_abi() != FOH_DSP_ABI) {
        std::fprintf(stderr, "abi mismatch\n");
        return 1;
    }
    const uint32_t sr = 48000;
    const size_t n = sr / 2;
    std::vector<float> x(n);
    for (size_t i = 0; i < n; ++i) {
        x[i] = 0.5f * std::sin(2.0f * static_cast<float>(M_PI) * 1000.0f * static_cast<float>(i)
                               / static_cast<float>(sr));
    }
    FohLevel lvl{};
    if (foh_levels(x.data(), x.size(), &lvl) != 0) {
        std::fprintf(stderr, "levels failed\n");
        return 1;
    }
    const float peak_db = 20.0f * std::log10(lvl.peak);
    if (peak_db < -7.0f || peak_db > -5.5f) {
        std::fprintf(stderr, "unexpected peak %.3f dB (lin %.5f)\n", peak_db, lvl.peak);
        return 1;
    }
    if (foh_peak_eq(x.data(), x.size(), sr, 1000.0f, 2.0f, -3.0f) != 0) {
        std::fprintf(stderr, "eq failed\n");
        return 1;
    }
    FohLevel after{};
    foh_levels(x.data(), x.size(), &after);
    if (after.peak >= lvl.peak) {
        std::fprintf(stderr, "cut did not reduce peak (%.5f -> %.5f)\n", lvl.peak, after.peak);
        return 1;
    }
    std::printf("foh_dsp selftest ok  peak=%.3f dBFS  after_cut=%.3f dBFS\n", peak_db,
                20.0f * std::log10(after.peak));
    return 0;
}
