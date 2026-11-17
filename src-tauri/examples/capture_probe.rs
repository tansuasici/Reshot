//! Dev probe: captures every monitor and writes PNGs to the given directory.
//! Usage: cargo run --release --example capture_probe -- <out-dir>
use std::time::Instant;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    for (i, m) in xcap::Monitor::all()
        .expect("list monitors")
        .into_iter()
        .enumerate()
    {
        let t = Instant::now();
        let img = m.capture_image().expect("capture");
        let cap = t.elapsed();
        println!(
            "monitor {i}: {} {}x{} at ({},{}) scale {} -> image {}x{} captured in {:?}",
            m.name().unwrap_or_default(),
            m.width().unwrap_or(0),
            m.height().unwrap_or(0),
            m.x().unwrap_or(0),
            m.y().unwrap_or(0),
            m.scale_factor().unwrap_or(1.0),
            img.width(),
            img.height(),
            cap
        );
        img.save(format!("{out}/monitor-{i}.png")).expect("save");
    }
}
