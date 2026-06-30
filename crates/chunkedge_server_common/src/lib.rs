#![doc = include_str!("../README.md")]

mod despawn;
mod uuid;

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use bevy_app::prelude::*;
use bevy_app::PluginsState;
use bevy_ecs::prelude::*;
use chunkedge_protocol::CompressionThreshold;
pub use despawn::*;

pub use crate::uuid::*;

/// Minecraft's standard ticks per second (TPS).
pub const DEFAULT_TPS: NonZeroU32 = match NonZeroU32::new(20) {
    Some(n) => n,
    None => unreachable!(),
};

#[derive(Clone, Resource)]
pub struct ServerSettings {
    /// The target ticks per second (TPS) of the server. This is the number of
    /// game updates that should occur in one second.
    ///
    /// On each game update (tick), the server is expected to update game logic
    /// and respond to packets from clients. Once this is complete, the server
    /// will sleep for any remaining time until a full tick duration has passed.
    ///
    /// Note that the official Minecraft client only processes packets at 20hz,
    /// so there is little benefit to a tick rate higher than the default 20.
    ///
    /// # Default Value
    ///
    /// [`DEFAULT_TPS`]
    pub tick_rate: NonZeroU32,
    /// The compression threshold to use for compressing packets. For a
    /// compression threshold of `Some(N)`, packets with encoded lengths >= `N`
    /// are compressed while all others are not. `None` disables compression
    /// completely.
    ///
    /// If the server is used behind a proxy on the same machine, you will
    /// likely want to disable compression.
    ///
    /// # Default Value
    ///
    /// Compression is enabled with an unspecified value. This value may
    /// change in future versions.
    pub compression_threshold: CompressionThreshold,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            tick_rate: DEFAULT_TPS,
            compression_threshold: CompressionThreshold(256),
        }
    }
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let settings = app
            .world_mut()
            .get_resource_or_insert_with(ServerSettings::default)
            .clone();

        app.insert_resource(Server {
            current_tick: 0,
            threshold: settings.compression_threshold,
            tick_rate: settings.tick_rate,
        });

        let tick_period = Duration::from_secs_f64(f64::from(settings.tick_rate.get()).recip());

        // Make the app loop forever at the configured TPS.
        app.set_runner(tick_loop_runner(tick_period));

        fn increment_tick_counter(mut server: ResMut<Server>) {
            server.current_tick += 1;
        }

        app.add_systems(Last, (increment_tick_counter, despawn_marked_entities));
    }
}

/// Builds the server's tick loop runner.
///
/// Behaves like vanilla Minecraft's scheduler: it targets an absolute per-tick
/// deadline so that `thread::sleep` overshoot is reclaimed on the following tick, keeping the long-run average
/// at exactly the configured TPS. When a tick runs long it catches up by
/// skipping the sleep; if it falls more than [`MAX_CATCH_UP`] behind it drops
/// the backlog instead of spiraling.
///
/// This replaces Bevy's [`ScheduleRunnerPlugin`](bevy_app::ScheduleRunnerPlugin),
/// which resets its clock every iteration and so never reclaims sleep overshoot.
fn tick_loop_runner(tick_period: Duration) -> impl FnOnce(App) -> AppExit {
    /// Maximum lag we try to make up before giving up and resetting the clock.
    /// Matches vanilla's "Can't keep up" threshold of 2 seconds.
    const MAX_CATCH_UP: Duration = Duration::from_secs(2);

    move |mut app: App| {
        // Drive plugins to readiness
        if app.plugins_state() != PluginsState::Cleaned {
            while app.plugins_state() == PluginsState::Adding {
                bevy_tasks::tick_global_task_pools_on_main_thread();
            }
            app.finish();
            app.cleanup();
        }

        let mut next_tick = Instant::now();
        loop {
            app.update();
            if let Some(exit) = app.should_exit() {
                return exit;
            }

            next_tick += tick_period;
            let now = Instant::now();
            if now < next_tick {
                // Ahead of schedule: wait until the next tick is due.
                std::thread::sleep(next_tick - now);
            } else if now - next_tick > MAX_CATCH_UP {
                // Too far behind: abandon the backlog so we don't death-spiral.
                next_tick = now;
            }
            // loop immediately to catch up.
        }
    }
}

/// Contains global server state accessible as a [`Resource`].
#[derive(Resource, Clone)]
pub struct Server {
    /// Incremented on every tick.
    current_tick: i64,
    threshold: CompressionThreshold,
    tick_rate: NonZeroU32,
}

impl Server {
    /// Returns the number of ticks that have elapsed since the server began.
    pub fn current_tick(&self) -> i64 {
        self.current_tick
    }

    /// Returns the server's [compression
    /// threshold](ServerSettings::compression_threshold).
    pub fn compression_threshold(&self) -> CompressionThreshold {
        self.threshold
    }

    // Returns the server's [tick rate](ServerPlugin::tick_rate).
    pub fn tick_rate(&self) -> NonZeroU32 {
        self.tick_rate
    }
}
