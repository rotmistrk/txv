//! Git stats chart demo — renders project line-count history as an inline image.

use std::process::Command;
use std::str::from_utf8;
use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::image_view::ImageView;

/// Generate the git stats chart demo.
pub(crate) fn make() -> Box<dyn View> {
    let mut iv = ImageView::new();
    let data = generate_chart();
    iv.set_image(data);
    iv.set_transform(ImageTransform::Fit);
    Box::new(iv)
}

/// Generate a simple bar chart as RGBA pixels showing line counts.
fn generate_chart() -> Arc<ImageData> {
    let stats = gather_line_stats();
    let w: u32 = 320;
    let h: u32 = 160;
    let mut pixels = vec![0u8; (w * h * 4) as usize];

    fill_background(&mut pixels);

    if stats.is_empty() {
        return Arc::new(ImageData::new(w, h, pixels));
    }

    let max_val = stats.iter().map(|s| s.code + s.test).max().unwrap_or(1).max(1);
    let bar_count = stats.len().min(w as usize);
    let bar_w = (w as usize / bar_count).max(1);
    let chart_h = h - 20;

    for (i, stat) in stats.iter().enumerate().take(bar_count) {
        let x_start = (i * bar_w) as u32;
        let code_h = (stat.code as u32 * chart_h / max_val as u32).min(chart_h);
        draw_bar(&mut pixels, w, x_start, bar_w as u32, chart_h - code_h, code_h, [70, 130, 230, 255]);
        let test_h = (stat.test as u32 * chart_h / max_val as u32).min(chart_h - code_h);
        if test_h > 0 {
            let y = chart_h - code_h - test_h;
            draw_bar(&mut pixels, w, x_start, bar_w as u32, y, test_h, [80, 200, 120, 255]);
        }
    }

    Arc::new(ImageData::new(w, h, pixels))
}

fn fill_background(pixels: &mut [u8]) {
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

fn gather_line_stats() -> Vec<LineStat> {
    let output = Command::new("git")
        .args(["log", "--oneline", "--reverse", "--format=%H"])
        .output()
        .ok();
    let Some(output) = output else {
        return sample_stats();
    };
    if !output.status.success() {
        return sample_stats();
    }
    let commits: Vec<&str> = from_utf8(&output.stdout).unwrap_or("").lines().collect();
    if commits.is_empty() {
        return sample_stats();
    }
    // Sample up to 40 evenly-spaced commits
    let step = (commits.len() / 40).max(1);
    let sampled: Vec<&str> = commits.iter().step_by(step).take(40).copied().collect();

    sampled.iter().map(|c| count_lines_at(c)).collect()
}

fn count_lines_at(commit: &str) -> LineStat {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", commit])
        .output()
        .ok();
    let Some(output) = output else {
        return LineStat { code: 0, test: 0 };
    };
    let text = from_utf8(&output.stdout).unwrap_or("");
    let rs_files: Vec<&str> = text.lines().filter(|l| l.ends_with(".rs")).collect();

    let mut code = 0usize;
    let mut test = 0usize;
    for file in &rs_files {
        let lines = count_file_lines(commit, file);
        if file.contains("test") || file.contains("gallery/tests") {
            test += lines;
        } else {
            code += lines;
        }
    }
    LineStat { code, test }
}

fn count_file_lines(commit: &str, file: &str) -> usize {
    Command::new("git")
        .args(["show", &format!("{commit}:{file}")])
        .output()
        .ok()
        .and_then(|o| from_utf8(&o.stdout).ok().map(|s| s.lines().count()))
        .unwrap_or(0)
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
        LineStat { code: 10000, test: 3600 },
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
