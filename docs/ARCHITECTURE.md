# Architecture

```
WAV / demo stems
        |
        v
 foh-core::analyze     spectrum, crest, K-ish loudness, ring, pearson
        |
        v
 foh-core::diagnose    stable codes
 foh-core::propose     capped MixMove list
        |
        v
 foh-control::safety   lock / rate / cap
        |
        +-- foh-mix strip -- analyze -- verify
        |
        +-- OSC journal (IPAD /sd / S-Series snapshot)
```

## Crates

| crate | job |
|-------|-----|
| foh-core | PCM, FFT, metrics, rules, verify |
| foh-mix | offline fader / peak EQ / comp / gate |
| foh-control | DiGiCo paths, OSC 1.0 encoder, plugin names, gate |
| foh-engine | CLI |

## C++

`cpp/foh_dsp` exports `foh_levels` and `foh_peak_eq` with a C ABI. Same RBJ
peak as `foh-mix`. Exists so a JUCE or console-side host can link without
pulling Rust. It is not a second product.

## Why no plugin host

A live VST3 rack that does not crash mid-show is a multi-year product
(Fourier transform.engine exists because of that). This repo analyzes and
proposes. Hosting Waves is out of scope until someone lands a sandboxed
host and an allowlist that has been run through a rehearsal.

## FFT

Radix-2 iterative, N = 4096, hop 1024, Hann. Magnitudes are `norm / N`.
The 1 kHz tone test in `crates/foh-core/tests/fft_tone.rs` is the check
that the bit-reversal and twiddles are not decorative.

## Standing-peak detector

Midband only (200 Hz–8 kHz). Score is `bin_magnitude * prominence_dB`.
A 70 Hz kick fundamental is peaky because its neighbors are empty; that
is not feedback. `crates/foh-core/tests/verify_gate.rs` locks that.

## Offline vs wire

EQ and dynamics run on the strip so verify can hear them. The public
DiGiCo OSC set in this repo does not include channel EQ, so those moves
never become packets. That is intentional. See `docs/CONTROL.md`.
