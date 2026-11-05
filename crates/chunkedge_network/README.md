# `chunkedge_network`

The plugin responsible for accepting connections and spawning clients.

This covers everything in the "handshaking", "status" and "login" stages of the protocol, before the main "play" stage begins. Support for proxies like [Velocity] and [BungeeCord] are implemented here.

ChunkEdge users can choose not to include `chunkedge_network` in their project. This could be useful for testing or using ChunkEdge as an integrated server in a client.

[Velocity]: https://papermc.io/software/velocity
[BungeeCord]: https://github.com/SpigotMC/BungeeCord

## Simulated network latency

`CHUNKEDGE_SIMULATE_LATENCY` selects an outgoing play-state stream profile:

| Value | Profile | Normal added delay | Batch jitter | Baseline target changes | Partial delivery interval | Suffix hold |
| --- | --- | --- | --- | --- | --- | --- |
| unset / `0` | Disabled | None | None | None | None | None |
| `1` | Low: regional EU, stable wired access | 15–40 ms | ±1 ms | 10–30 s | 10–30 s | 1–5 ms |
| `2` | Medium: EU ↔ eastern US, stable wired access | 80–110 ms | ±2 ms | 10–30 s | 5–15 s | 3–10 ms |
| `3` | High: EU ↔ Australia, stable access | 270–330 ms | ±3 ms | 15–45 s | 5–15 s | 5–15 ms |
| `4` | Variable regional cellular / busy wireless, stress case | 35–85 ms | ±10 ms | 2–6 s | 2–6 s | 10–40 ms |

The normal range includes jitter; it is not the baseline plus jitter again.
The initial baseline is the midpoint of the range and drifts linearly toward a
random target inside it, leaving room for the batch jitter. Split intervals are
sampled in wall-clock seconds, rather than using a per-tick probability. At an
event, roughly 25–75% of a batch goes first and the remainder follows after the
suffix hold. Tiny one-byte batches are not split; idle periods do not trigger
catch-up events. A hold can push delivery beyond the normal range.

Level 4 is a separate variability stress profile, not a higher-distance tier.
Healthy cellular networks can have lower latency than levels 2 or 3. Levels 2 and
3 deliberately have the same split cadence: distance alone does not imply worse
wireless quality or more frequent stalls.

For the moving wall, run from the repository root:

```sh
CHUNKEDGE_SIMULATE_LATENCY=2 nix develop -c cargo run --example cow_sphere
```

**Compatibility:** the previous `1` setting (roughly 90 ms) is now closest to `2`.
Unset or `0` disables simulation. Other values emit a warning and disable it.
The setting is read when each play-state connection is created; changing it
requires restarting the server to affect subsequent connections.

### Research and interpretation

The geographic ranges are illustrative, informed by measured RTTs rather than
universal definitions of low/medium/high. IBM's November 2026 inter-region table
reports about 13 ms London–Frankfurt, 75 ms London–Washington DC, and 293 ms
London–Sydney. These are cloud-backbone measurements, not residential promises;
the regional preset allows extra access/routing delay. See
[IBM's measured RTT dashboard](https://cloud.ibm.com/docs/vpc?topic=vpc-network-latency-dashboard).

Ofcom's 2025 UK mobile study reports median response times of 15–21 ms for 5G
and 18–23 ms for 4G across operators. This is why cellular is modeled as a
separate variability case, rather than assuming every mobile connection has
hundreds of milliseconds of latency. See
[Ofcom Mobile Matters 2025, response time section](https://www.ofcom.org.uk/siteassets/resources/documents/research-and-data/telecoms-research/mobile-matters/2025/mobile-matters-2025.pdf?v=400314).

The jitter amplitudes, drift intervals, split intervals, and suffix holds are
explicit modeling choices, not distributions derived from these sources. Their
purpose is to contrast stable and variable delivery while keeping nearby batches
correlated. Realistic values for a specific ISP/route would require a measured
latency trace, including behavior under load.

### Timing and limitations

Because only outbound traffic is delayed, the whole nominal RTT allowance is
applied there. For example, level 2 adds roughly 80–110 ms to round-trip ping
on an otherwise fast connection. It does not reproduce a symmetric connection
with half the delay in each direction. Existing latency, retained suffixes,
backpressure, and application scheduling can raise observed ping.

Batches are pipelined with absolute deadlines, so propagation delay does not
become a per-batch throughput cap. Newer bytes cannot overtake earlier bytes.
Splits can occur inside packet lengths, compressed data, or encrypted data.
The scheduler and socket writer share the existing asynchronous writer task;
ECS ticks are unaffected. The delivery queue holds at most 16 fragments, plus
in-flight data and the existing bounded outgoing byte channel and pending output.
This accommodates several ticks in flight at the normal 20 Hz, including level 3.

This remains application-level scheduling before TCP, not IP packet emulation.
The OS can split/combine writes, so client reads need not match simulated
fragments. See [TCP segmentation](https://www.rfc-editor.org/rfc/rfc9293.html#section-3.7).
The simulator does not model bandwidth, MTU, congestion control, ACKs, actual
loss/retransmission, or network-level reordering. Incoming traffic and login are
unchanged. Under backpressure or unusually high batch rates the pipeline can
fill, bytes can merge in the outgoing channel, and delay can exceed the preset.
