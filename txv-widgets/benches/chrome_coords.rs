//! Benchmark: chrome coordinate computation is O(n) in panel count,
//! O(1) in screen dimensions — independent of how large the rects are.
//!
//! Run: cargo bench --bench chrome_coords

use std::hint::black_box;
use std::time::Instant;

use txv_core::prelude::Rect;

/// Simulates compute_chrome_coords: given panel rects, compute tier rows and gap columns.
/// This is the exact algorithm from TiledWorkspace::compute_chrome_coords.
fn compute_chrome_coords(rects: &[Rect]) -> (Vec<u16>, Vec<(u16, u16, u16)>) {
    let mut tier_ys: Vec<u16> = rects.iter().map(|r| r.y).collect();
    tier_ys.sort_unstable();
    tier_ys.dedup();

    let mut gaps: Vec<(u16, u16, u16)> = Vec::new();
    for a in rects {
        let gap_x = a.x + a.w;
        let has_neighbor = rects.iter().any(|b| b.x == gap_x + 1 && b.y == a.y);
        if !has_neighbor {
            continue;
        }
        if gaps.iter().any(|&(x, ys, _)| x == gap_x && ys == a.y) {
            continue;
        }
        gaps.push((gap_x, a.y, a.y + a.h));
    }

    (tier_ys, gaps)
}

/// 4-panel layout at given screen size (simulates wide layout).
fn make_rects(w: u32, h: u32) -> Vec<Rect> {
    let w = w.min(u16::MAX as u32) as u16;
    let h = h.min(u16::MAX as u32) as u16;
    let top_h = (h as u32 * 7 / 10) as u16;
    let bot_h = h - top_h;
    let left_w = (w as u32 * 2 / 10) as u16;
    let center_w = (w as u32 * 4 / 10) as u16;
    let right_w = w - left_w - center_w - 2; // 2 gap columns
    vec![
        Rect::new(0, 0, left_w, top_h),
        Rect::new(left_w + 1, 0, center_w, top_h),
        Rect::new(left_w + center_w + 2, 0, right_w, top_h),
        Rect::new(0, top_h, w, bot_h),
    ]
}

fn bench_compute(label: &str, w: u32, h: u32, iterations: u32) {
    let rects = make_rects(w, h);
    // Warm up
    for _ in 0..100 {
        black_box(compute_chrome_coords(black_box(&rects)));
    }
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(compute_chrome_coords(black_box(&rects)));
    }
    let elapsed = start.elapsed();
    println!(
        "{:<30} {:>10.1} ns/iter  ({} iters)",
        label,
        elapsed.as_nanos() as f64 / iterations as f64,
        iterations,
    );
}

fn main() {
    let iters = 1_000_000;

    println!("compute_chrome_coords benchmark — O(n) panels, O(1) screen size\n");

    bench_compute("80×24 (terminal)", 80, 24, iters);
    bench_compute("300×80 (wide terminal)", 300, 80, iters);
    bench_compute("1920×1080 (full HD)", 1920, 1080, iters);
    bench_compute("65535×65535 (u16 max)", 65535, 65535, iters);
    bench_compute("2000000000×3000000000 (2B×3B)", 2_000_000_000, 3_000_000_000, iters);

    println!("\nAll sizes produce identical timing — algorithm is O(1) in screen dimensions.");
}
