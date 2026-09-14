# Limits

What this tree does. What it refuses to pretend to do.

## Known

- Offline PCM in (WAV 16-bit PCM or 32-bit float, or the built-in problem mix).
- Per-stem peak, RMS, crest, DC, activity, ISO-ish octave energy, K-ish loudness.
- Standing-peak detector restricted to 200 Hz–8 kHz on open-mic roles.
- Kick/bass Pearson and 63/125 occupancy rules.
- Bounded mix moves: fader ≤ 3 dB, EQ ≤ 4 dB, Q in [0.7, 8.0].
- Offline strip: fader, RBJ peak, one-band compressor, gate.
- Public DiGiCo OSC 1.0 encode for fader / mute / solo / phantom / snapshot / macro.
- Safety gate: locked names, rate limit, second cap pass.
- Verify reject on after-clip and worse howl.

## Known gaps

- No Dante, MADI, SoundGrid, JACK, or console tap.
- No live socket from `foh-engine`. Print only.
- No channel EQ or dynamics OSC. Companion's public module does not document those paths as first-class.
- No Waves SuperRack ProLink. No Fourier transform.engine host.
- Fader law `0.75 = 0 dB` is a display approximation, not a desk calibration.
- K-weighting is a two-biquad stand-in, not ITU-R BS.1770.
- Demo stems are sines. They prove the loop, not a mix.

## Flip conditions

A live send path is only justified when all of these are true:

1. The OSC address is documented in `docs/CONTROL.md` with the Companion (or vendor) source.
2. Verify has a reject path for the failure mode that move can cause.
3. A human enable flag defaults off.
4. The desk position is read back or the operator accepts open-loop.

Until then the journal is the product.
