# AI FOH Engine

Listen, reason, mix, verify. Offline first. Console last.

Open source. Rust analysis + C++ DSP kernel. Speaks the public DiGiCo OSC
subset (iPad / `/sd/` / S-Series snapshots). Does not host Waves or Fourier.

```
cargo test --workspace
cargo run -p foh-engine
```

On a FUSE or noexec checkout:

```
export CARGO_TARGET_DIR=/tmp/ai-foh-target
bash scripts/test.sh
```

## Loop

1. Listen — per-stem peak, RMS, crest, octave energy, standing-peak detector (200 Hz–8 kHz), kick/bass Pearson.
2. Reason — deterministic codes (`docs/RULES.md`). No network model.
3. Mix — apply capped moves on an offline strip. Print OSC for faders only, using `maps/demo.map` (or `--map`).
4. Verify — re-measure. Overs or a worse howl reject the pass. Rejected passes are not journaled.

## Demo

Built-in stems. No audio files required.

Typical pass: vocal standing peak at 2496 Hz, kick/bass fight at 63 Hz,
narrow cut on the vocal, bass 63 Hz cut, verify Accept.

## What this is not

- A DiGiCo session parser
- SuperRack ProLink
- Fourier transform.engine control
- A Dante / SoundGrid tap
- An LLM on the fader

Waves and Fourier cannot both be enabled on current Quantum integration.
Neither protocol is public. Plugin names live in `foh-control::plugins` as
an allowlist for later, not as a wire.

## Layout

```
crates/foh-core       analyze / diagnose / propose / verify / FFT
crates/foh-mix        offline strip
crates/foh-control    OSC 1.0 + safety + journal
crates/foh-engine     CLI
cpp/foh_dsp           C ABI: levels + RBJ peak
maps/demo.map         name to desk channel
docs/                 architecture, rules, control surface, safety
```

## Map file

```
# name  desk_input  role
kick 1 kick
bass 2 bass
vox 12 vocal
```

`foh-engine --map maps/demo.map`

## License

MIT OR Apache-2.0.
