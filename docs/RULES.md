# Diagnosis codes

All codes are stable strings. Reasoners must not invent new ones in a
show-critical path without adding a verify check.

| code | severity | meaning | default move |
|------|----------|---------|--------------|
| clip_risk | Crit | peak > −1 dBFS | fader −2.5 dB |
| hot_peak | Warn | peak > −3 dBFS | fader −1.5 dB |
| lr_clip | Crit | summed LR peak > −1 dBFS | fader down on hottest stem |
| feedback_risk | Crit | open mic, standing peak ≥ 10 dB vs neighbors, 200 Hz–8 kHz | narrow cut ≤ 3.5 dB |
| ring_mode | Warn | same detector, 8–10 dB | narrow cut ≤ 2 dB |
| vocal_buried | Warn | vocal RMS < −28 dBFS while active | fader +2, optional 3.2 kHz +1.5 |
| vocal_masked | Warn | vocal presence below competing stems | same |
| kick_bass_63 | Warn | both occupy the 63 Hz octave | bass cut 63 Hz |
| kick_bass_125 | Warn | both occupy 125 Hz | kick cut 125 Hz |
| boxy_low_mids | Warn | 250 Hz hotter than low end on a low-end source | cut 250 Hz |
| kick_bass_corr | Info | Pearson |r| > 0.85 | no move; context only |

Caps (enforced in `propose` and again in `SafetyGate`):

- fader Δ ≤ 3.0 dB
- EQ Δ ≤ 4.0 dB
- Q in [0.7, 8.0]
- talkback / announce locked
- ≥ 250 ms between moves on one channel

Verify rejects the pass if the after mix overs (peak > 0 dBFS) or howl
prominence on an open mic rises by more than 2 dB.
