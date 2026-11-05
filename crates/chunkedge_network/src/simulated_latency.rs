//! Illustrative regional, long-haul, and variable-access links before socket writes.

use std::io;
use std::time::Duration;

use bytes::BytesMut;
use rand::{RngExt, SeedableRng};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::time::Instant;

use crate::byte_channel::ByteReceiver;

/// Presets describe added RTT, concentrated on the outgoing direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Level {
    Regional,
    Transatlantic,
    Intercontinental,
    VariableCellular,
}

impl Level {
    fn parse(value: &str) -> Result<Option<Self>, &'static str> {
        match value.trim() {
            "0" => Ok(None),
            "1" => Ok(Some(Self::Regional)),
            "2" => Ok(Some(Self::Transatlantic)),
            "3" => Ok(Some(Self::Intercontinental)),
            "4" => Ok(Some(Self::VariableCellular)),
            _ => Err(
                "expected 0 (off), 1 (regional), 2 (transatlantic), 3 (intercontinental), or 4 (variable cellular)",
            ),
        }
    }

    pub(crate) fn from_env() -> Option<Self> {
        let value = std::env::var_os("CHUNKEDGE_SIMULATE_LATENCY")?;
        match value
            .to_str()
            .map_or(Err("value is not valid Unicode"), Self::parse)
        {
            Ok(level) => level,
            Err(reason) => {
                tracing::warn!("Invalid CHUNKEDGE_SIMULATE_LATENCY: {reason}; simulation disabled");
                None
            }
        }
    }

    fn settings(self) -> Settings {
        // Delay ranges are informed by measured RTTs; jitter and event timing are
        // illustrative tuning, not distributions fitted to those measurements.
        // Keep geographic distance separate from wireless variability.
        match self {
            Self::Regional => Settings {
                delay_ms: (15.0, 40.0),
                jitter_ms: 1.0,
                drift_secs: (10, 30),
                split_secs: (10, 30),
                hold_ms: (1, 5),
            },
            Self::Transatlantic => Settings {
                delay_ms: (80.0, 110.0),
                jitter_ms: 2.0,
                drift_secs: (10, 30),
                split_secs: (5, 15),
                hold_ms: (3, 10),
            },
            Self::Intercontinental => Settings {
                delay_ms: (270.0, 330.0),
                jitter_ms: 3.0,
                drift_secs: (15, 45),
                split_secs: (5, 15),
                hold_ms: (5, 15),
            },
            Self::VariableCellular => Settings {
                delay_ms: (35.0, 85.0),
                jitter_ms: 10.0,
                drift_secs: (2, 6),
                split_secs: (2, 6),
                hold_ms: (10, 40),
            },
        }
    }
}

#[derive(Clone, Copy)]
struct Settings {
    delay_ms: (f64, f64),
    jitter_ms: f64,
    drift_secs: (u64, u64),
    split_secs: (u64, u64),
    hold_ms: (u64, u64),
}

impl Settings {
    fn target(self, rng: &mut impl rand::Rng) -> f64 {
        rng.random_range(self.delay_ms.0 + self.jitter_ms..=self.delay_ms.1 - self.jitter_ms)
    }
}

struct Delivery {
    at: Instant,
    bytes: BytesMut,
}

struct Profile {
    settings: Settings,
    rng: rand::rngs::StdRng,
    drift_start: Instant,
    drift_end: Instant,
    start_ms: f64,
    target_ms: f64,
    next_split: Instant,
    last_delivery: Instant,
}

impl Profile {
    fn new(now: Instant, seed: [u8; 32], level: Level) -> Self {
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let settings = level.settings();
        Self {
            settings,
            drift_start: now,
            drift_end: now
                + Duration::from_secs(
                    rng.random_range(settings.drift_secs.0..=settings.drift_secs.1),
                ),
            start_ms: (settings.delay_ms.0 + settings.delay_ms.1) / 2.0,
            target_ms: settings.target(&mut rng),
            next_split: now
                + Duration::from_secs(
                    rng.random_range(settings.split_secs.0..=settings.split_secs.1),
                ),
            last_delivery: now,
            rng,
        }
    }

    fn schedule(&mut self, now: Instant, mut bytes: BytesMut) -> (Delivery, Option<Delivery>) {
        while now >= self.drift_end {
            self.drift_start = self.drift_end;
            self.drift_end += Duration::from_secs(
                self.rng
                    .random_range(self.settings.drift_secs.0..=self.settings.drift_secs.1),
            );
            self.start_ms = self.target_ms;
            self.target_ms = self.settings.target(&mut self.rng);
        }

        let progress = (now - self.drift_start).as_secs_f64()
            / (self.drift_end - self.drift_start).as_secs_f64();
        let baseline_ms = self.start_ms + (self.target_ms - self.start_ms) * progress;
        let delay_ms = (baseline_ms
            + self
                .rng
                .random_range(-self.settings.jitter_ms..=self.settings.jitter_ms))
        .clamp(self.settings.delay_ms.0, self.settings.delay_ms.1);
        let at = (now + Duration::from_secs_f64(delay_ms / 1000.0)).max(self.last_delivery);

        let tail = if now >= self.next_split && bytes.len() > 1 {
            let min = (bytes.len() / 4).max(1);
            let max = (bytes.len() - bytes.len() / 4).min(bytes.len() - 1);
            let prefix_len = self.rng.random_range(min..=max);
            self.next_split = now
                + Duration::from_secs(
                    self.rng
                        .random_range(self.settings.split_secs.0..=self.settings.split_secs.1),
                );
            Some(Delivery {
                at: at
                    + Duration::from_millis(
                        self.rng
                            .random_range(self.settings.hold_ms.0..=self.settings.hold_ms.1),
                    ),
                bytes: bytes.split_off(prefix_len),
            })
        } else {
            None
        };

        // Later bytes cannot overtake a held suffix, even if their sampled delay is lower.
        self.last_delivery = tail.as_ref().map_or(at, |tail| tail.at);
        (Delivery { at, bytes }, tail)
    }
}

pub(crate) async fn write(
    writer: &mut (impl AsyncWrite + Unpin),
    receiver: &mut ByteReceiver,
    level: Level,
) -> io::Result<()> {
    write_with_profile(
        writer,
        receiver,
        Profile::new(Instant::now(), rand::random(), level),
    )
    .await
}

async fn write_with_profile(
    writer: &mut (impl AsyncWrite + Unpin),
    receiver: &mut ByteReceiver,
    mut profile: Profile,
) -> io::Result<()> {
    // Pipeline batches: propagation delay must not become a per-batch throughput cap.
    // Sixteen fragments accommodate the high-latency preset at 20 ticks/sec,
    // including occasional splits. The queue and byte channel keep memory bounded.
    let (sender, mut deliveries) = tokio::sync::mpsc::channel::<Delivery>(16);
    let schedule = async move {
        while let Ok(bytes) = receiver.recv_async().await {
            let (prefix, tail) = profile.schedule(Instant::now(), bytes);
            if sender.send(prefix).await.is_err() {
                return Ok(());
            }
            if let Some(tail) = tail
                && sender.send(tail).await.is_err()
            {
                return Ok(());
            }
        }
        // Closing the sender lets the writer drain scheduled bytes at EOF.
        drop(sender);
        io::Result::Ok(())
    };
    let deliver = async {
        while let Some(delivery) = deliveries.recv().await {
            tokio::time::sleep_until(delivery.at).await;
            writer.write_all(&delivery.bytes).await?;
        }
        io::Result::Ok(())
    };

    // Both futures live in the existing writer task and are cancelled together.
    tokio::try_join!(schedule, deliver)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    use super::*;
    use crate::byte_channel::byte_channel;

    const LEVELS: [Level; 4] = [
        Level::Regional,
        Level::Transatlantic,
        Level::Intercontinental,
        Level::VariableCellular,
    ];

    #[test]
    fn parses_disabled_and_all_presets_and_rejects_invalid_values() {
        assert_eq!(Level::parse("0"), Ok(None));
        for (value, level) in ["1", "2", "3", "4"].into_iter().zip(LEVELS) {
            assert_eq!(Level::parse(value), Ok(Some(level)));
        }
        assert_eq!(Level::parse(" 2 "), Ok(Some(Level::Transatlantic)));
        for value in ["", "-1", "5", "true", "1.0"] {
            assert!(Level::parse(value).is_err());
        }
    }

    #[test]
    fn each_preset_drifts_within_bounds_without_accumulating_per_batch() {
        let now = Instant::now();
        for level in LEVELS {
            let mut profile = Profile::new(now, [42; 32], level);
            let settings = level.settings();
            let mut previous_ms: Option<f64> = None;
            // 120 seconds at the usual 20 Hz; high latency must remain a delay,
            // not turn into an ever-growing queue of serialized batch sleeps.
            for tick in 0..2400 {
                let queued_at = now + Duration::from_millis(tick * 50);
                let (delivery, tail) = profile.schedule(queued_at, BytesMut::from(&b"x"[..]));
                assert!(tail.is_none());
                let delay_ms = (delivery.at - queued_at).as_secs_f64() * 1000.0;
                assert!((settings.delay_ms.0..=settings.delay_ms.1).contains(&delay_ms));
                if let Some(previous_ms) = previous_ms {
                    assert!((delay_ms - previous_ms).abs() < 2.0 * settings.jitter_ms + 1.0);
                }
                previous_ms = Some(delay_ms);
            }
        }
    }

    #[test]
    fn each_preset_retains_suffixes_at_its_cadence_without_reordering() {
        let now = Instant::now();
        for level in LEVELS {
            let mut profile = Profile::new(now, [42; 32], level);
            let settings = level.settings();
            let mut previous_split = now;
            let mut count = 0;
            for tick in 0..2400 {
                let queued_at = now + Duration::from_millis(tick * 50);
                let input = BytesMut::from(&b"distinct bytes in a batch"[..]);
                let (prefix, tail) = profile.schedule(queued_at, input.clone());
                if let Some(tail) = tail {
                    assert!(
                        (Duration::from_secs(settings.split_secs.0)
                            ..=Duration::from_secs(settings.split_secs.1))
                            .contains(&(queued_at - previous_split))
                    );
                    assert!(
                        (Duration::from_millis(settings.hold_ms.0)
                            ..=Duration::from_millis(settings.hold_ms.1))
                            .contains(&(tail.at - prefix.at))
                    );
                    let mut joined = prefix.bytes;
                    joined.extend_from_slice(&tail.bytes);
                    assert_eq!(joined, input);
                    let (next, _) = profile.schedule(queued_at, BytesMut::from(&b"new"[..]));
                    assert!(next.at >= tail.at);
                    previous_split = queued_at;
                    count += 1;
                }
            }
            assert!(count >= 2);
        }
    }

    #[tokio::test]
    async fn all_presets_preserve_the_complete_byte_stream() {
        for level in LEVELS {
            let expected: Vec<u8> = (0_u32..16_384).flat_map(u32::to_le_bytes).collect();
            let input = expected.clone();
            let (mut sender, mut receiver) = byte_channel(16_384);
            // Force partial socket writes, independently of simulated fragmentation.
            let (mut writer, mut reader) = tokio::io::duplex(17);
            let mut profile = Profile::new(Instant::now(), [42; 32], level);
            profile.next_split = Instant::now();
            let produce = async move {
                for batch in input.chunks(8192) {
                    sender
                        .send_async(BytesMut::from(batch))
                        .await
                        .map_err(|_| io::Error::other("byte channel closed"))?;
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                drop(sender);
                io::Result::Ok(())
            };
            let transmit = async {
                write_with_profile(&mut writer, &mut receiver, profile).await?;
                writer.shutdown().await
            };
            let receive = async {
                let mut received = Vec::new();
                reader.read_to_end(&mut received).await?;
                io::Result::Ok(received)
            };
            let ((), (), received) = tokio::time::timeout(Duration::from_secs(10), async {
                tokio::try_join!(produce, transmit, receive)
            })
            .await
            .expect("simulated stream did not drain")
            .expect("simulated stream failed");
            assert_eq!(received, expected);
        }
    }
}
