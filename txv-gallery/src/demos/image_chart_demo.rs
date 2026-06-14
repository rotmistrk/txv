//! Git stats chart demo — renders project line-count history as an inline image.
//! Uses a single `git log --numstat` call for all data (fast even over SSH).

use std::process::Command;
use std::str::from_utf8;
use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::image_view::ImageView;

pub(crate) fn make() -> Box<dyn View> {
    let mut iv = ImageView::new();
    let data = generate_chart();
    iv.set_image(data);
    iv.set_transform(ImageTransform::Fit);
    Box::new(iv)
}

fn generate_chart() -> Arc<ImageData> {
    let stats = gather_line_stats();
    let w: u32 = 320;
    let h: u32 = 160;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    fill_background(&mut pixels, w, h);

    if stats.is_empty() {
        return Arc::new(ImageData::new(w, h, pixels));
    }

    let max_val = stats.iter().map(|s| s.code + s.test).max().unwrap_or(1).max(1);
    let bar_count = stats.len().min(w as usize);
    let bar_w = (w as usize / bar_count).max(1);
    let chart_h = h - 20;

    for (i, stat) in stats.iter().enumerate().take(bar_count) {
        let x = (i * bar_w) as u32;
        let bw = bar_w as u32;
        let code_h = (stat.code as u32 * chart_h / max_val as u32).min(chart_h);
        draw_bar(&mut pixels, w, x, bw, chart_h - code_h, code_h, [70, 130, 230, 255]);
        let test_h = (stat.test as u32 * chart_h / max_val as u32).min(chart_h - code_h);
        if test_h > 0 {
            draw_bar(
                &mut pixels,
                w,
                x,
                bw,
                chart_h - code_h - test_h,
                test_h,
                [80, 200, 120, 255],
            );
        }
    }

    Arc::new(ImageData::new(w, h, pixels))
}

fn fill_background(pixels: &mut [u8], _w: u32, _h: u32) {
    for i in (0..pixels.len()).step_by(4) {
        pixels[i] = 30;
        pixels[i + 1] = 30;
        pixels[i + 2] = 35;
        pixels[i + 3] = 255;
    }
}

struct LineStat {
    code: usize,
    test: usize,
}

/// Gather cumulative line counts using ONE `git log --numstat` command.
fn gather_line_stats() -> Vec<LineStat> {
    let output = Command::new("git")
        .args(["log", "--reverse", "--numstat", "--format=%H"])
        .output()
        .ok();
    let Some(output) = output else {
        return sample_stats();
    };
    if !output.status.success() {
        return sample_stats();
    }
    let text = from_utf8(&output.stdout).unwrap_or("");
    parse_numstat(text)
}

/// Parse git log --numstat output into cumulative stats per commit.
fn parse_numstat(text: &str) -> Vec<LineStat> {
    let mut all_commits: Vec<LineStat> = Vec::new();
    let mut code: i64 = 0;
    let mut test: i64 = 0;

    for line in text.lines() {
        if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            // Commit hash — record snapshot
            all_commits.push(LineStat {
                code: code.max(0) as usize,
                test: test.max(0) as usize,
            });
        } else if let Some((add, del, path)) = parse_numstat_line(line) {
            if !path.ends_with(".rs") {
                continue;
            }
            let net = add as i64 - del as i64;
            if is_test_file(path) {
                test += net;
            } else {
                code += net;
            }
        }
    }
    // Final state
    all_commits.push(LineStat {
        code: code.max(0) as usize,
        test: test.max(0) as usize,
    });

    // Sample down to ~30 points
    if all_commits.len() <= 30 {
        return all_commits;
    }
    let step = all_commits.len() / 30;
    all_commits.into_iter().step_by(step).take(30).collect()
}

fn parse_numstat_line(line: &str) -> Option<(usize, usize, &str)> {
    let mut parts = line.split('\t');
    let add_str = parts.next()?;
    let del_str = parts.next()?;
    let path = parts.next()?;
    let add: usize = add_str.parse().ok()?;
    let del: usize = del_str.parse().ok()?;
    Some((add, del, path))
}

fn is_test_file(path: &str) -> bool {
    path.contains("test") || path.contains("gallery/tests")
}

fn sample_stats() -> Vec<LineStat> {
    vec![
        LineStat { code: 500, test: 100 },
        LineStat { code: 1200, test: 300 },
        LineStat { code: 2500, test: 600 },
        LineStat { code: 4000, test: 1200 },
        LineStat { code: 5500, test: 1800 },
        LineStat { code: 7000, test: 2400 },
        LineStat { code: 8500, test: 3000 },
        LineStat {
            code: 10000,
            test: 3600,
        },
    ]
}

fn draw_bar(pixels: &mut [u8], img_w: u32, x: u32, w: u32, y: u32, h: u32, color: [u8; 4]) {
    for row in y..y + h {
        for col in x..x + w.saturating_sub(1) {
            let i = ((row * img_w + col) * 4) as usize;
            if i + 3 < pixels.len() {
                pixels[i..i + 4].copy_from_slice(&color);
            }
        }
    }
}
