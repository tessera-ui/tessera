#!/usr/bin/env rust-script
//!
//! Analyze tessera profiler JSONL output and summarize hot components.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! anyhow = "1.0"
//! clap = { version = "4.0", features = ["derive"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    bin_name = "rust-script scripts/analyze-profiler.rs"
)]
struct Cli {
    /// Path to the profiler JSONL file.
    path: PathBuf,

    /// Show top N entries per section.
    #[arg(long, default_value_t = 20)]
    top: usize,

    /// Minimum samples per component to include in top lists.
    #[arg(long, default_value_t = 1)]
    min_count: u64,

    /// Skip lines that fail JSON parsing.
    #[arg(long)]
    skip_invalid: bool,
}

#[derive(Deserialize)]
struct FrameRecord {
    #[allow(dead_code)]
    frame: u64,
    render_time_ns: Option<u128>,
    frame_total_ns: Option<u128>,
    components: Vec<ComponentRecord>,
}

#[derive(Deserialize)]
struct ComponentRecord {
    #[allow(dead_code)]
    id: String,
    fn_name: Option<String>,
    layout_cache_hit: Option<bool>,
    phases: PhaseDurations,
    children: Vec<ComponentRecord>,
}

#[derive(Deserialize)]
struct PhaseDurations {
    build_ns: Option<u128>,
    measure_ns: Option<u128>,
    input_ns: Option<u128>,
}

#[derive(Default)]
struct Summary {
    frames: u64,
    frame_total_sum: u128,
    frame_total_min: Option<u128>,
    frame_total_max: Option<u128>,
    frame_total_count: u64,
    render_total: u128,
    render_count: u64,
    build_total_sum: u128,
    build_total_min: Option<u128>,
    build_total_max: Option<u128>,
    build_total_count: u64,
    measure_total_sum: u128,
    measure_total_min: Option<u128>,
    measure_total_max: Option<u128>,
    measure_total_count: u64,
    input_total_sum: u128,
    input_total_min: Option<u128>,
    input_total_max: Option<u128>,
    input_total_count: u64,
    node_count: u64,
    cache_hit: u64,
    cache_miss: u64,
    cache_unknown: u64,
}

#[derive(Default, Clone)]
struct Stats {
    count: u64,
    build_ns: u128,
    measure_ns: u128,
    input_ns: u128,
    cache_hit: u64,
    cache_miss: u64,
    cache_unknown: u64,
}

impl Stats {
    fn total_ns(&self) -> u128 {
        self.build_ns + self.measure_ns + self.input_ns
    }

    fn hit_rate(&self) -> Option<f64> {
        let denom = self.cache_hit + self.cache_miss;
        if denom == 0 {
            None
        } else {
            Some(self.cache_hit as f64 / denom as f64)
        }
    }
}

#[derive(Default, Clone, Copy)]
struct PhaseTotals {
    build: u128,
    measure: u128,
    input: u128,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file = File::open(&cli.path)
        .with_context(|| format!("failed to open profiler file at {}", cli.path.display()))?;
    let reader = BufReader::new(file);

    let mut summary = Summary::default();
    let mut stats_by_name: HashMap<String, Stats> = HashMap::new();

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result.with_context(|| format!("failed to read line {}", line_idx + 1))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<FrameRecord>(trimmed) {
            Ok(frame) => process_frame(frame, &mut summary, &mut stats_by_name),
            Err(_) => {
                let value = serde_json::from_str::<serde_json::Value>(trimmed).ok();
                let object = value.as_ref().and_then(|v| v.as_object());
                let is_header = object
                    .and_then(|obj| obj.get("format"))
                    .and_then(|val| val.as_str())
                    .is_some_and(|format| format == "tessera-profiler");
                if is_header {
                    continue;
                }
                if object
                    .and_then(|obj| obj.get("frame"))
                    .and_then(|val| val.as_u64())
                    .is_some()
                {
                    bail!("invalid frame record at line {}", line_idx + 1);
                }
                if !cli.skip_invalid {
                    bail!("unrecognized JSON line at {}", line_idx + 1);
                }
            }
        }
    }

    if summary.frames == 0 {
        bail!("no frame records found");
    }

    print_summary(&summary);
    print_top_sections(&stats_by_name, cli.top, cli.min_count);

    Ok(())
}

fn process_frame(
    frame: FrameRecord,
    summary: &mut Summary,
    stats_by_name: &mut HashMap<String, Stats>,
) {
    summary.frames += 1;

    let mut frame_totals = PhaseTotals::default();
    for component in &frame.components {
        let _ =
            accumulate_component_exclusive(component, summary, stats_by_name, &mut frame_totals);
    }

    summary.build_total_sum += frame_totals.build;
    summary.build_total_count += 1;
    summary.build_total_min = Some(
        summary
            .build_total_min
            .map_or(frame_totals.build, |v| v.min(frame_totals.build)),
    );
    summary.build_total_max = Some(
        summary
            .build_total_max
            .map_or(frame_totals.build, |v| v.max(frame_totals.build)),
    );

    summary.measure_total_sum += frame_totals.measure;
    summary.measure_total_count += 1;
    summary.measure_total_min = Some(
        summary
            .measure_total_min
            .map_or(frame_totals.measure, |v| v.min(frame_totals.measure)),
    );
    summary.measure_total_max = Some(
        summary
            .measure_total_max
            .map_or(frame_totals.measure, |v| v.max(frame_totals.measure)),
    );

    summary.input_total_sum += frame_totals.input;
    summary.input_total_count += 1;
    summary.input_total_min = Some(
        summary
            .input_total_min
            .map_or(frame_totals.input, |v| v.min(frame_totals.input)),
    );
    summary.input_total_max = Some(
        summary
            .input_total_max
            .map_or(frame_totals.input, |v| v.max(frame_totals.input)),
    );

    if let Some(total) = frame.frame_total_ns {
        summary.frame_total_sum += total;
        summary.frame_total_count += 1;
        summary.frame_total_min = Some(summary.frame_total_min.map_or(total, |v| v.min(total)));
        summary.frame_total_max = Some(summary.frame_total_max.map_or(total, |v| v.max(total)));
    }

    if let Some(render) = frame.render_time_ns {
        summary.render_total += render;
        summary.render_count += 1;
    }
}

fn accumulate_component_exclusive(
    component: &ComponentRecord,
    summary: &mut Summary,
    stats_by_name: &mut HashMap<String, Stats>,
    frame_totals: &mut PhaseTotals,
) -> PhaseTotals {
    summary.node_count += 1;

    let mut children_inclusive = PhaseTotals::default();
    for child in &component.children {
        let child_inclusive =
            accumulate_component_exclusive(child, summary, stats_by_name, frame_totals);
        children_inclusive.build += child_inclusive.build;
        children_inclusive.measure += child_inclusive.measure;
        children_inclusive.input += child_inclusive.input;
    }

    let inclusive = PhaseTotals {
        build: component.phases.build_ns.unwrap_or(0),
        measure: component.phases.measure_ns.unwrap_or(0),
        input: component.phases.input_ns.unwrap_or(0),
    };

    let exclusive = PhaseTotals {
        build: inclusive.build.saturating_sub(children_inclusive.build),
        measure: inclusive.measure.saturating_sub(children_inclusive.measure),
        input: inclusive.input.saturating_sub(children_inclusive.input),
    };

    frame_totals.build += exclusive.build;
    frame_totals.measure += exclusive.measure;
    frame_totals.input += exclusive.input;

    let name = component
        .fn_name
        .as_deref()
        .unwrap_or("<unknown>")
        .to_string();
    let entry = stats_by_name.entry(name).or_default();
    entry.count += 1;
    entry.build_ns += exclusive.build;
    entry.measure_ns += exclusive.measure;
    entry.input_ns += exclusive.input;

    match component.layout_cache_hit {
        Some(true) => {
            entry.cache_hit += 1;
            summary.cache_hit += 1;
        }
        Some(false) => {
            entry.cache_miss += 1;
            summary.cache_miss += 1;
        }
        None => {
            entry.cache_unknown += 1;
            summary.cache_unknown += 1;
        }
    }

    inclusive
}

fn print_summary(summary: &Summary) {
    println!("Profiler summary");
    println!("- Frames: {}", summary.frames);
    if summary.frame_total_count > 0 {
        let avg_frame = summary.frame_total_sum as f64 / summary.frame_total_count as f64;
        let min_frame = summary.frame_total_min.unwrap_or(0);
        let max_frame = summary.frame_total_max.unwrap_or(0);
        println!(
            "- Frame total (avg/min/max): {} / {} / {} ms",
            format_ms(avg_frame),
            format_ms(min_frame as f64),
            format_ms(max_frame as f64)
        );
    }
    if summary.render_count > 0 {
        let avg_render = summary.render_total as f64 / summary.render_count as f64;
        println!("- Render time avg: {} ms", format_ms(avg_render));
    }
    if summary.build_total_count > 0 {
        let avg_build = summary.build_total_sum as f64 / summary.build_total_count as f64;
        let min_build = summary.build_total_min.unwrap_or(0);
        let max_build = summary.build_total_max.unwrap_or(0);
        println!(
            "- Build total (exclusive avg/min/max): {} / {} / {} ms",
            format_ms(avg_build),
            format_ms(min_build as f64),
            format_ms(max_build as f64)
        );
    }
    if summary.measure_total_count > 0 {
        let avg_measure = summary.measure_total_sum as f64 / summary.measure_total_count as f64;
        let min_measure = summary.measure_total_min.unwrap_or(0);
        let max_measure = summary.measure_total_max.unwrap_or(0);
        println!(
            "- Measure total (exclusive avg/min/max): {} / {} / {} ms",
            format_ms(avg_measure),
            format_ms(min_measure as f64),
            format_ms(max_measure as f64)
        );
    }
    if summary.input_total_count > 0 {
        let avg_input = summary.input_total_sum as f64 / summary.input_total_count as f64;
        let min_input = summary.input_total_min.unwrap_or(0);
        let max_input = summary.input_total_max.unwrap_or(0);
        println!(
            "- Input total (exclusive avg/min/max): {} / {} / {} ms",
            format_ms(avg_input),
            format_ms(min_input as f64),
            format_ms(max_input as f64)
        );
    }
    let cache_total = summary.cache_hit + summary.cache_miss;
    if cache_total > 0 {
        let rate = summary.cache_hit as f64 / cache_total as f64;
        println!(
            "- Layout cache hit rate: {}% (hit {}, miss {}, unknown {})",
            format_pct(rate),
            summary.cache_hit,
            summary.cache_miss,
            summary.cache_unknown
        );
    } else {
        println!(
            "- Layout cache hit rate: n/a (unknown {})",
            summary.cache_unknown
        );
    }
    println!("- Components counted: {}", summary.node_count);
    println!();
}

fn print_top_sections(stats: &HashMap<String, Stats>, top: usize, min_count: u64) {
    let mut rows: Vec<(&String, &Stats)> = stats
        .iter()
        .filter(|(_, stat)| stat.count >= min_count)
        .collect();

    rows.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.measure_ns));
    print_section("Top by measure_ns (exclusive)", &rows, top, |s| {
        s.measure_ns
    });

    rows.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.build_ns));
    print_section("Top by build_ns (exclusive)", &rows, top, |s| s.build_ns);

    rows.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.input_ns));
    print_section("Top by input_ns (exclusive)", &rows, top, |s| s.input_ns);

    rows.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.total_ns()));
    print_section("Top by total_ns (exclusive)", &rows, top, |s| s.total_ns());
}

fn print_section<F>(title: &str, rows: &[(&String, &Stats)], top: usize, value: F)
where
    F: Fn(&Stats) -> u128,
{
    println!("{title}");
    for (idx, (name, stat)) in rows.iter().take(top).enumerate() {
        let total_ns = value(stat);
        let avg_ns = if stat.count == 0 {
            0.0
        } else {
            total_ns as f64 / stat.count as f64
        };
        let hit_rate = stat.hit_rate();
        let hit_rate_display = hit_rate
            .map(format_pct)
            .unwrap_or_else(|| "n/a".to_string());
        println!(
            "{:>2}. {:<40} total {} ms, avg {} us, count {}, hit {} ({} / {} / {})",
            idx + 1,
            truncate(name, 40),
            format_ms(total_ns as f64),
            format_us(avg_ns),
            stat.count,
            hit_rate_display,
            stat.cache_hit,
            stat.cache_miss,
            stat.cache_unknown
        );
    }
    println!();
}

fn format_ms(value_ns: f64) -> String {
    format!("{:.3}", value_ns / 1_000_000.0)
}

fn format_us(value_ns: f64) -> String {
    format!("{:.2}", value_ns / 1_000.0)
}

fn format_pct(rate: f64) -> String {
    format!("{:.1}", rate * 100.0)
}

fn truncate(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        text.to_string()
    } else if limit <= 3 {
        "...".to_string()
    } else {
        let cutoff = limit - 3;
        format!("{}...", &text[..cutoff])
    }
}
