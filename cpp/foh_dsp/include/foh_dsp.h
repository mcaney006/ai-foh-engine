#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float peak;
    float rms;
    float dc;
} FohLevel;

int foh_levels(const float *samples, size_t n, FohLevel *out);
int foh_peak_eq(float *samples, size_t n, uint32_t sr, float freq_hz, float q, float gain_db);

#ifdef __cplusplus
}
#endif
