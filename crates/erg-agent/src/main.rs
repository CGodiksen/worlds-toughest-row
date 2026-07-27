//! Connect to a Concept2 PM5 over BLE, dump its services and characteristics, then stream the
//! General Status characteristic (0x0031) and print the live elapsed-time / distance.

use std::time::{Duration, Instant};

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use uuid::Uuid;

/// Concept2 PM UUIDs share the base `CE06xxxx-43E5-11E4-916C-0800200C9A66`. Only the 16-bit "short"
/// field varies (0x0031 = General Status).
fn pm5_uuid(short: u16) -> Uuid {
    Uuid::parse_str(&format!("ce06{short:04x}-43e5-11e4-916c-0800200c9a66"))
        .expect("valid PM5 UUID")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no Bluetooth adapter found"))?;

    println!("Scanning for a Concept2 PM5 (up to 15s)...");

    adapter.start_scan(ScanFilter::default()).await?;
    let pm = find_pm5(&adapter, Duration::from_secs(30)).await?;
    adapter.stop_scan().await?;

    let name = pm
        .properties()
        .await?
        .and_then(|p| p.local_name)
        .unwrap_or_else(|| "PM5".into());

    println!("Found \"{name}\". Connecting...");

    pm.connect().await?;
    pm.discover_services().await?;

    println!("\nConnected. Services / characteristics:");

    for service in pm.services() {
        println!("  service {}", service.uuid);
        for ch in &service.characteristics {
            println!("    char {}  {:?}", ch.uuid, ch.properties);
        }
    }

    // Subscribe to General Status (0x0031) and print what streams.
    let general_status = pm5_uuid(0x0031);
    let ch = pm
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == general_status)
        .ok_or_else(|| anyhow::anyhow!("General Status characteristic (0x0031) not found"))?;
    pm.subscribe(&ch).await?;

    println!("\nSubscribed to General Status. Do a Just Row — Ctrl+C to quit.\n");

    let mut notifications = pm.notifications().await?;
    let mut first = true;
    while let Some(n) = notifications.next().await {
        if n.uuid != general_status {
            continue;
        }
        if first {
            // Dump the raw frame once so we can sanity-check field offsets.
            println!(
                "(raw first frame, {} bytes: {:02x?})",
                n.value.len(),
                n.value
            );
            first = false;
        }
        if let Some((secs, meters, workout_state, rowing_state)) = parse_general_status(&n.value) {
            println!(
                "t={secs:7.2}s  dist={meters:8.1}m  workout_state={workout_state}  rowing_state={rowing_state}"
            );
        }
    }

    println!("\nNotification stream ended — PM5 disconnected or went to sleep.");

    Ok(())
}

/// Poll the adapter's discovered peripherals until one advertises a name starting with "PM5", or
/// `timeout` elapses.
async fn find_pm5(adapter: &Adapter, timeout: Duration) -> anyhow::Result<Peripheral> {
    let start = Instant::now();
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
        if start.elapsed() >= timeout {
            anyhow::bail!(
                "No PM5 found."
            );
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Decode the leading fields of the 0x0031 General Status frame.
/// `Elapsed time = bytes 0..3 in 0.01 s` and `distance = bytes 3..6 in 0.1 m`.
fn parse_general_status(d: &[u8]) -> Option<(f64, f64, u8, u8)> {
    if d.len() < 11 {
        return None;
    }

    let elapsed = d[0] as u32 | (d[1] as u32) << 8 | (d[2] as u32) << 16;
    let distance = d[3] as u32 | (d[4] as u32) << 8 | (d[5] as u32) << 16;
    let workout_state = d[8];
    let rowing_state = d[9];

    Some((
        elapsed as f64 * 0.01,
        distance as f64 * 0.1,
        workout_state,
        rowing_state,
    ))
}
