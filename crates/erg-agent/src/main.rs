//! Watches a Concept2 PM5 over BLE and logs each rowing session to the tracker's API. One entry
//! per session (distance climbs, then stops for a while).
//!
//! For each cycle we get a fresh Bluetooth adapter, scan, wait for the erg to advertise (it does
//! once you start rowing), connect, POST each finished session, disconnect, and cool down so the
//! erg can sleep before we scan again. A fresh adapter each cycle keeps btleplug's WinRT device
//! cache from wedging across reconnects.

use std::time::{Duration, Instant};

use anyhow::anyhow;
use api_types::AdvanceRequest;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use reqwest::Client;
use uuid::Uuid;

/// The tracker's "log a row" endpoint, the same one the web form posts to.
const API_URL: &str = "http://127.0.0.1:8080/api/entries";

/// No rowing for this long ends a session and logs it.
const SESSION_IDLE: Duration = Duration::from_secs(30);

/// No rowing for this long drops the BLE link so the erg can sleep.
const DISCONNECT_IDLE: Duration = Duration::from_secs(90);

/// Wait this long after disconnecting before scanning again, so the erg actually sleeps instead of
/// being reconnected immediately. Must exceed the erg's own idle-sleep (~4 min). Rows are far
/// enough apart that this never misses one.
const COOLDOWN: Duration = Duration::from_secs(300);

/// Ignore sessions shorter than this.
const MIN_SESSION_M: f64 = 15.0;

/// Distance-change threshold in meters (frames report 0.1 m units).
const EPS: f64 = 0.05;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    loop {
        let manager = Manager::new().await?;
        let adapter = first_adapter(&manager).await?;
        adapter.start_scan(ScanFilter::default()).await?;

        println!("Waiting for the erg...");
        let pm = find_pm5(&adapter).await?;

        println!("Erg detected: connecting...");
        if let Err(e) = run_session(&pm, &client).await {
            eprintln!("Session ended: {e}");
        }

        pm.disconnect().await.ok();
        println!("Disconnected: cooling down so the erg can sleep.\n");

        drop(adapter);
        drop(manager);

        tokio::time::sleep(COOLDOWN).await;
    }
}

/// The first available Bluetooth adapter.
async fn first_adapter(manager: &Manager) -> anyhow::Result<Adapter> {
    manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No Bluetooth adapter found."))
}

/// Poll until a PM5 is advertising (which it does once you start rowing).
async fn find_pm5(adapter: &Adapter) -> anyhow::Result<Peripheral> {
    loop {
        for p in adapter.peripherals().await? {
            let is_pm5 = p
                .properties()
                .await?
                .and_then(|props| props.local_name)
                .is_some_and(|n| n.starts_with("PM5"));

            if is_pm5 {
                return Ok(p);
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Connect, then stream distance, and POST each finished session. Returns when idle long enough to
/// disconnect (or the link drops).
async fn run_session(pm: &Peripheral, client: &Client) -> anyhow::Result<()> {
    let status = connect(pm).await?;
    let mut notifications = pm.notifications().await?;
    let mut tracker = SessionTracker::new();
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            frame = notifications.next() => {
                let Some(n) = frame else { return Ok(()); }; // Link dropped.
                if n.uuid == status.uuid {
                    if let Some(dist) = parse_distance(&n.value) {
                        if let Some(meters) = tracker.observe(dist) {
                            log_row(client, meters).await;
                        }
                    }
                }
            }
            _ = ticker.tick() => {
                if let Some(meters) = tracker.close_if_idle() {
                    log_row(client, meters).await;
                }

                if tracker.idle_for(DISCONNECT_IDLE) {
                    return Ok(());
                }
            }
        }
    }
}

/// Connect and subscribe to General Status with a few quick retries (BLE connects reliably once
/// you're rowing, but the first attempt can still glitch).
async fn connect(pm: &Peripheral) -> anyhow::Result<Characteristic> {
    let mut last_err = None;
    for attempt in 1..=3 {
        match try_connect(pm).await {
            Ok(ch) => return Ok(ch),
            Err(e) => {
                eprintln!("Connect attempt {attempt} failed: {e}");
                last_err = Some(e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    Err(last_err.expect("at least one attempt"))
}

async fn try_connect(pm: &Peripheral) -> anyhow::Result<Characteristic> {
    pm.connect().await?;
    pm.discover_services().await?;

    let status = pm
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == pm5_uuid(0x0031))
        .ok_or_else(|| anyhow!("General Status characteristic missing."))?;

    pm.subscribe(&status).await?;
    Ok(status)
}

/// POST a finished session. A failure just logs, the manual form is the backstop, and the server
/// runs alongside, so this rarely fails.
async fn log_row(client: &Client, meters: i32) {
    let body = AdvanceRequest {
        meters: Some(meters),
    };

    match client.post(API_URL).json(&body).send().await {
        Ok(r) if r.status().is_success() => println!("Logged {meters} m."),
        Ok(r) => eprintln!("Server rejected row: {}.", r.status()),
        Err(e) => eprintln!("Failed to log {meters} m: {e}."),
    }
}

/// Decode cumulative distance (bytes 3..6, 0.1 m) from a General Status frame.
fn parse_distance(d: &[u8]) -> Option<f64> {
    if d.len() < 6 {
        return None;
    }

    let raw = d[3] as u32 | (d[4] as u32) << 8 | (d[5] as u32) << 16;
    Some(raw as f64 * 0.1)
}

/// Concept2 PM UUID for a 16-bit short id (0x0031 = General Status).
fn pm5_uuid(short: u16) -> Uuid {
    Uuid::parse_str(&format!("ce06{short:04x}-43e5-11e4-916c-0800200c9a66"))
        .expect("valid PM5 UUID")
}

/// Turns the PM5's cumulative distance into per-session meters. A session opens when distance
/// climbs and closes after `SESSION_IDLE` of no progress (or when the monitor is reset, dropping
/// distance back down). Timing uses the wall clock because the PM5's own elapsed time freezes when
/// you stop.
struct SessionTracker {
    /// The last distance we saw.
    last_distance: f64,
    /// The distance at the start of the current session.
    session_start: f64,
    /// Whether we're currently in a session.
    active: bool,
    /// When we last saw progress.
    last_progress: Instant,
}

impl SessionTracker {
    fn new() -> Self {
        Self {
            last_distance: 0.0,
            session_start: 0.0,
            active: false,
            last_progress: Instant::now(),
        }
    }

    /// Feed a cumulative-distance reading. Returns the meters of a session closed by a monitor
    /// reset (distance dropping), if any.
    fn observe(&mut self, dist: f64) -> Option<i32> {
        if dist > self.last_distance + EPS {
            if !self.active {
                self.active = true;
                self.session_start = self.last_distance;
            }

            self.last_distance = dist;
            self.last_progress = Instant::now();

            None
        } else if dist < self.last_distance - EPS {
            let closed = self.close();

            self.last_distance = dist;
            self.session_start = dist;

            closed
        } else {
            None
        }
    }

    /// Close and return the current session if it's gone quiet for `SESSION_IDLE`.
    fn close_if_idle(&mut self) -> Option<i32> {
        if self.active && self.last_progress.elapsed() >= SESSION_IDLE {
            self.close()
        } else {
            None
        }
    }

    /// Whether there's no open session and no progress for `d`.
    fn idle_for(&self, d: Duration) -> bool {
        !self.active && self.last_progress.elapsed() >= d
    }

    /// End the current session, returning its meters if it clears the floor.
    fn close(&mut self) -> Option<i32> {
        if !self.active {
            return None;
        }
        self.active = false;

        let meters = self.last_distance - self.session_start;
        self.session_start = self.last_distance;
        (meters >= MIN_SESSION_M).then_some(meters.round() as i32)
    }
}
