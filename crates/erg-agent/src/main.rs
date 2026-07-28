//! Watches a Concept2 PM5 over BLE and logs each row to the tracker's API. One connection is one
//! row: the erg wakes when you start pulling, we follow its cumulative distance and log the total
//! once it stops climbing.
//!
//! Each cycle we get a fresh Bluetooth adapter, scan, wait for the erg to advertise (it does once
//! you start rowing), connect, log the row, disconnect, and cool down so the erg can sleep before
//! scanning again. A fresh adapter each cycle keeps btleplug's WinRT device cache from wedging
//! across reconnects, and a bounded scan window means a scan that never wakes (started before the
//! Bluetooth stack was ready at boot, or dropped by a driver glitch) gets recycled instead of
//! polling a dead watcher forever.

use std::time::{Duration, Instant};

use anyhow::anyhow;
use api_types::AdvanceRequest;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use reqwest::Client;
use uuid::Uuid;

/// The tracker's "log a row" endpoint, the same one the web form posts to.
const API_URL: &str = "http://127.0.0.1:4800/api/entries";

/// No progress for this long means the row is finished.
const IDLE: Duration = Duration::from_secs(30);

/// Wait this long after disconnecting before scanning again, so the erg actually sleeps instead of
/// being reconnected immediately. Must exceed the erg's own idle-sleep (~4 min). Rows are far
/// enough apart that this never misses one.
const COOLDOWN: Duration = Duration::from_secs(300);

/// Recycle the adapter if no erg appears within one scan window. A scan can wedge silently on
/// Windows (started before the Bluetooth stack was ready at boot, or dropped by an HCI glitch).
/// Rebuilding the adapter gives a fresh WinRT device object instead of polling a dead watcher
/// forever.
const SCAN_WINDOW: Duration = Duration::from_secs(120);

/// Back off this long after a Bluetooth error before retrying, so a flaky adapter is not hammered.
const RETRY_DELAY: Duration = Duration::from_secs(5);

/// Ignore sessions shorter than this.
const MIN_METERS: f64 = 15.0;

/// Concept2 PM5 General Status characteristic. Cumulative distance is reported here.
const GENERAL_STATUS: Uuid = Uuid::from_u128(0xce06_0031_43e5_11e4_916c_0800_200c_9a66);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    // Never exit. Any Bluetooth error just means recycle and retry.
    loop {
        if let Err(e) = run_cycle(&client).await {
            eprintln!("Cycle error: {e}.");
            tokio::time::sleep(RETRY_DELAY).await;
        }
    }
}

/// One scan-to-cooldown cycle with a fresh adapter. Returns `Ok` after logging a row and after an
/// empty scan window (so the caller rescans immediately with a new adapter), and propagates any
/// Bluetooth error so the caller can back off and retry with a fresh adapter.
async fn run_cycle(client: &Client) -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let adapter = first_adapter(&manager).await?;
    adapter.start_scan(ScanFilter::default()).await?;

    println!("Waiting for the erg...");
    let found = find_pm5(&adapter).await?;
    adapter.stop_scan().await.ok();

    let Some(pm) = found else {
        // Scan window elapsed with no erg. Drop the adapter and let the caller build a fresh one.
        return Ok(());
    };

    println!("Erg detected: connecting...");
    match row_distance(&pm).await {
        Ok(Some(meters)) => log_row(client, meters).await,
        Ok(None) => {}
        Err(e) => eprintln!("Session error: {e}."),
    }

    pm.disconnect().await.ok();
    println!("Disconnected: cooling down so the erg can sleep.\n");

    tokio::time::sleep(COOLDOWN).await;
    Ok(())
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

/// Poll until a PM5 is advertising (which it does once you start rowing), or `None` if none appears
/// within `SCAN_WINDOW` so the caller can recycle the adapter.
async fn find_pm5(adapter: &Adapter) -> anyhow::Result<Option<Peripheral>> {
    let deadline = Instant::now() + SCAN_WINDOW;
    while Instant::now() < deadline {
        for p in adapter.peripherals().await? {
            let is_pm5 = p
                .properties()
                .await?
                .and_then(|props| props.local_name)
                .is_some_and(|n| n.starts_with("PM5"));

            if is_pm5 {
                return Ok(Some(p));
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(None)
}

/// Connect and follow cumulative distance. Returns the meters rowed once the erg goes quiet for
/// `IDLE` (or the link drops), or `None` if it stayed below the floor.
async fn row_distance(pm: &Peripheral) -> anyhow::Result<Option<i32>> {
    pm.connect().await?;
    pm.discover_services().await?;

    let status = pm
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == GENERAL_STATUS)
        .ok_or_else(|| anyhow!("General Status characteristic missing."))?;

    pm.subscribe(&status).await?;

    let mut notifications = pm.notifications().await?;
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    let mut last_distance = 0.0;
    let mut last_progress = Instant::now();

    loop {
        tokio::select! {
            frame = notifications.next() => {
                let Some(n) = frame else { break }; // Link dropped.
                if n.uuid == status.uuid && let Some(current_distance) = parse_distance(&n.value) && current_distance > last_distance {
                    last_distance = current_distance;
                    last_progress = Instant::now();
                }
            }
            _ = ticker.tick() => {
                if last_progress.elapsed() >= IDLE {
                    break;
                }
            }
        }
    }

    Ok((last_distance >= MIN_METERS).then_some(last_distance.round() as i32))
}

/// POST a finished row. A failure just logs, the manual form is the backstop, and the server runs
/// alongside, so this rarely fails.
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
