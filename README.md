# AI FOH Engine

Listen → Reason → Mix → Verify.

Open-source front-of-house analysis. Rust rules + C++ DSP kernel. Speaks the
public DiGiCo OSC subset used by Companion / the console iPad. Does not host
Waves. Does not host Fourier. Does not put an LLM on a fader.

```
make ci
```

On a FUSE or noexec checkout the Makefile already points `CARGO_TARGET_DIR`
at `/tmp/ai-foh-target`.

## What a pass does

1. **Listen** — per-stem peak, RMS, crest, octave energy, standing-peak
   detector (200 Hz–8 kHz, open-mic roles only), kick/bass Pearson.
2. **Reason** — deterministic codes in `docs/RULES.md`. No network model.
3. **Mix** — apply capped moves on an offline strip. Print OSC for the
   public fader/mute/snapshot/macro set only.
4. **Verify** — re-measure. Overs or a worse howl reject the pass.
   Rejected passes are not journaled.

Built-in problem mix (no WAV required):

```
cargo run -p foh-engine -- --map maps/demo.map
```

Expected on that mix: vocal standing peak ~2496 Hz, kick/bass fight at
63 Hz, narrow cuts, `VERIFY Accept`.

WAV ingest:

```
cargo run -p foh-engine -- --stem vox=vox.wav --stem kick=kick.wav --map maps/demo.map
```

16-bit PCM or 32-bit float WAVE. First channel of each file becomes the stem.

## Layout

```
crates/foh-core       analyze / diagnose / propose / verify / FFT
crates/foh-mix        offline strip (fader, RBJ peak, comp, gate)
crates/foh-control    OSC 1.0 encoder, safety gate, plugin name allowlist
crates/foh-engine     CLI
cpp/foh_dsp           C ABI: levels + RBJ peak
maps/demo.map         name → desk channel + role
docs/                 architecture, rules, control surface, safety, limits
```

Zero crate dependencies. The sandbox that built this hit crates.io 502s and
a noexec FUSE volume; std is the contract.

## What this is not

- A DiGiCo session parser
- SuperRack ProLink
- Fourier transform.engine control
- A Dante / SoundGrid tap
- An automatic mix that goes to the desk without an operator

Waves and Fourier cannot both be enabled on current Quantum integration.
Neither protocol is public. Plugin names live in `foh-control::plugins` as
an allowlist for later, not as a wire.

## Map file

```
# name  desk_input  role
kick 1 kick
bass 2 bass
vox 12 vocal
```

Desk channels are 1-based input numbers as used by `/Input_Channels/{n}/fader`.

## License

MIT OR Apache-2.0.

DiGiCo, Waves, SuperRack, and Fourier are trademarks of their owners.
This project is not affiliated with them.
