# Console control — what is wired, what is not

## DiGiCo OSC (Known)

Source: Bitfocus Companion module `companion-module-digico-osc` (community,
not a DiGiCo SDK). Paths below are what that module sends.

| Set | Prefix | Example |
|-----|--------|---------|
| IPAD (default) | none | `/Input_Channels/{n}/fader` |
| OSC | `/sd` | `/sd/Input_Channels/{n}/fader` |
| S-Series | `/digico` | `/digico/snapshots/fire` only |

Supported in this repo:

- fader (float 0..1)
- mute / solo / phantom
- snapshot fire / next / prev
- macro press (0-based on the wire; user numbers are 1-based)

Not in that public set, therefore not encoded here:

- channel EQ bands
- dynamics
- aux sends as first-class objects (Companion documents some mute paths only)
- plugin parameters

Fader law used for dry-run display: 0.0 = −inf, 0.75 = 0 dB, 1.0 = +10 dB.
That is an approximation so a log line is inspectable. It is not a DiGiCo
calibration. Do not treat the float as a measured desk position without
reading the desk back.

## Waves SuperRack / Fourier transform.engine (Known gap)

Quantum software treats Fourier integration and Waves integration as mutually
exclusive. Neither vendor publishes an open parameter protocol this project
can speak. `foh-control::plugins` is a name-and-range allowlist only.

## Audio ingest (Known gap)

No Dante, SoundGrid, MADI, or JACK tap in this tree. Input is WAV or the
built-in problem mix. Analysis math does not care how the buffer arrived.
