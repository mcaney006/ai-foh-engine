#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FOH_DSP_ABI 1

typedef struct {
    float peak;
    float rms;
    float dc;
} FohLevel;

int foh_dsp_abi(void);

/* Peak / RMS / DC on a mono buffer. Returns 0 on success. */
int foh_levels(const float *samples, size_t n, FohLevel *out);

/* In-place RBJ peaking EQ. Returns 0 on success. */
int foh_peak_eq(float *samples, size_t n, uint32_t sr, float freq_hz, float q, float gain_db);

#ifdef __cplusplus
}
#endif
