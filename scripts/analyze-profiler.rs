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
//! comfy-table = "7.1"
//! owo-colors = "4.1"
//! csv = "1.3"
//! ```

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Row, Table, presets::UTF8_FULL};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

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

    /// Export full per-component aggregated stats to a CSV file.
    #[arg(long)]
    csv: Option<PathBuf>,
}

#[derive(Deserialize)]
struct FrameRecord {
    #[allow(dead_code)]
    frame: u64,
    render_time_ns: Option<u128>,
    build_tree_time_ns: Option<u128>,
    draw_time_ns: Option<u128>,
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
    frame_total: DurationStats,
    render: DurationStats,
    build_tree: DurationStats,
    draw: DurationStats,
    build: DurationStats,
    measure: DurationStats,
    input: DurationStats,
    unaccounted: DurationStats,
    node_count: u64,
    cache_hit: u64,
    cache_miss: u64,
    cache_unknown: u64,
}

#[derive(Default, Clone, Copy)]
struct DurationStats {
    total: u128,
    min: Option<u128>,
    max: Option<u128>,
    count: u64,
}

impl DurationStats {
    fn record(&mut self, value: u128) {
        self.total += value;
        self.min = Some(self.min.map_or(value, |min| min.min(value)));
        self.max = Some(self.max.map_or(value, |max| max.max(value)));
        self.count += 1;
    }
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

    if let Some(path) = &cli.csv {
        export_csv(path, &stats_by_name)?;
        println!("{} {}", "Wrote CSV:".green(), path.display());
    }

    print_summary(&summary);
    print_top_sections(&stats_by_name, cli.top, cli.min_count);

    Ok(())
}

#[derive(Serialize)]
struct CsvRow {
    component: String,
    count: u64,

    build_total_ns: u128,
    build_total_ms: f64,
    build_avg_us: f64,

    measure_total_ns: u128,
    measure_total_ms: f64,
    measure_avg_us: f64,

    input_total_ns: u128,
    input_total_ms: f64,
    input_avg_us: f64,

    total_total_ns: u128,
    total_total_ms: f64,
    total_avg_us: f64,

    cache_hit: u64,
    cache_miss: u64,
    cache_unknown: u64,
    cache_hit_rate: Option<f64>,
}

fn export_csv(path: &PathBuf, stats: &HashMap<String, Stats>) -> Result<()> {
    let mut rows: Vec<(&String, &Stats)> = stats.iter().collect();
    rows.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.total_ns()));

    let file = File::create(path)
        .with_context(|| format!("failed to create CSV output file at {}", path.display()))?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(file);

    for (name, stat) in rows {
        let count = stat.count;
        let total_ns = stat.total_ns();

        writer
            .serialize(CsvRow {
                component: name.to_string(),
                count,

                build_total_ns: stat.build_ns,
                build_total_ms: stat.build_ns as f64 / 1_000_000.0,
                build_avg_us: avg_ns(stat.build_ns, count) / 1_000.0,

                measure_total_ns: stat.measure_ns,
                measure_total_ms: stat.measure_ns as f64 / 1_000_000.0,
                measure_avg_us: avg_ns(stat.measure_ns, count) / 1_000.0,

                input_total_ns: stat.input_ns,
                input_total_ms: stat.input_ns as f64 / 1_000_000.0,
                input_avg_us: avg_ns(stat.input_ns, count) / 1_000.0,

                total_total_ns: total_ns,
                total_total_ms: total_ns as f64 / 1_000_000.0,
                total_avg_us: avg_ns(total_ns, count) / 1_000.0,

                cache_hit: stat.cache_hit,
                cache_miss: stat.cache_miss,
                cache_unknown: stat.cache_unknown,
                cache_hit_rate: stat.hit_rate(),
            })
            .with_context(|| format!("failed to write CSV row for component '{name}'"))?;
    }

    writer
        .flush()
        .with_context(|| format!("failed to flush CSV output file at {}", path.display()))?;
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

    summary.build.record(frame_totals.build);
    summary.measure.record(frame_totals.measure);
    summary.input.record(frame_totals.input);

    if let Some(total) = frame.frame_total_ns {
        summary.frame_total.record(total);
        if let Some(build_tree) = frame.build_tree_time_ns {
            summary.build_tree.record(build_tree);
        }
        if let Some(draw) = frame.draw_time_ns {
            summary.draw.record(draw);
        }
        if let Some(render) = frame.render_time_ns {
            summary.render.record(render);
            let accounted = frame_totals.build + frame_totals.measure + frame_totals.input + render;
            summary.unaccounted.record(total.saturating_sub(accounted));
        }
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

fn add_duration_row(
    table: &mut Table,
    label: &str,
    stat: DurationStats,
    show_bounds: bool,
    color: Option<Color>,
) {
    if stat.count == 0 {
        return;
    }
    let bounds = show_bounds.then(|| stat.min.zip(stat.max)).flatten();
    let value = |ns: f64| Cell::new(format!("{} ms", format_ms(ns)));
    let cells = [
        Cell::new(label),
        value(avg_ns(stat.total, stat.count)),
        bounds.map_or_else(|| Cell::new(""), |(min, _)| value(min as f64)),
        bounds.map_or_else(|| Cell::new(""), |(_, max)| value(max as f64)),
    ];
    table.add_row(Row::from(
        cells
            .into_iter()
            .map(|cell| match color {
                Some(color) => cell.fg(color),
                None => cell,
            })
            .collect::<Vec<_>>(),
    ));
}

fn print_summary(summary: &Summary) {
    println!("{}", "Profiler summary".bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(Row::from(vec![
            Cell::new("Metric").fg(Color::Cyan),
            Cell::new("Avg").fg(Color::Cyan),
            Cell::new("Min").fg(Color::Cyan),
            Cell::new("Max").fg(Color::Cyan),
        ]));

    table.add_row(Row::from(vec![
        Cell::new("Frames"),
        Cell::new(summary.frames.to_string()),
        Cell::new(""),
        Cell::new(""),
    ]));

    for (label, stat, show_bounds, color) in [
        ("Frame cost", summary.frame_total, true, None),
        ("Build cost", summary.build_tree, true, None),
        ("Draw/compute cost", summary.draw, true, None),
        ("Render cost", summary.render, false, None),
        ("Build total", summary.build, true, None),
        ("Measure total", summary.measure, true, None),
        ("Input total", summary.input, true, None),
        (
            "Unaccounted cost",
            summary.unaccounted,
            true,
            Some(Color::DarkGrey),
        ),
    ] {
        add_duration_row(&mut table, label, stat, show_bounds, color);
    }

    let cache_total = summary.cache_hit + summary.cache_miss;
    if cache_total > 0 {
        let rate = summary.cache_hit as f64 / cache_total as f64;
        table.add_row(Row::from(vec![
            Cell::new("Layout cache hit rate"),
            Cell::new(format!(
                "{}% (hit {}, miss {}, unknown {})",
                format_pct(rate),
                summary.cache_hit,
                summary.cache_miss,
                summary.cache_unknown
            )),
            Cell::new(""),
            Cell::new(""),
        ]));
    } else {
        table.add_row(Row::from(vec![
            Cell::new("Layout cache hit rate"),
            Cell::new(format!("n/a (unknown {})", summary.cache_unknown)),
            Cell::new(""),
            Cell::new(""),
        ]));
    }

    table.add_row(Row::from(vec![
        Cell::new("Components counted"),
        Cell::new(summary.node_count.to_string()),
        Cell::new(""),
        Cell::new(""),
    ]));

    println!("{table}");
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
    println!("{}", title.bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(Row::from(vec![
            Cell::new("#").fg(Color::Cyan),
            Cell::new("Component").fg(Color::Cyan),
            Cell::new("Total (ms)").fg(Color::Cyan),
            Cell::new("Avg (us)").fg(Color::Cyan),
            Cell::new("Count").fg(Color::Cyan),
            Cell::new("Hit rate").fg(Color::Cyan),
        ]));

    for (idx, (name, stat)) in rows.iter().take(top).enumerate() {
        let total_ns = value(stat);
        let avg_ns = avg_ns(total_ns, stat.count);
        let hit_rate_display = stat
            .hit_rate()
            .map(format_pct)
            .unwrap_or_else(|| "n/a".to_string());
        table.add_row(Row::from(vec![
            Cell::new((idx + 1).to_string()),
            Cell::new(truncate(name, 40)),
            Cell::new(format_ms(total_ns as f64)),
            Cell::new(format_us(avg_ns)),
            Cell::new(stat.count.to_string()),
            Cell::new(format!(
                "{} ({} / {} / {})",
                hit_rate_display, stat.cache_hit, stat.cache_miss, stat.cache_unknown
            )),
        ]));
    }

    println!("{table}\n");
}

fn format_ms(value_ns: f64) -> String {
    format!("{:.3}", value_ns / 1_000_000.0)
}

fn avg_ns(total: u128, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn format_us(value_ns: f64) -> String {
    format!("{:.2}", value_ns / 1_000.0)
}

fn format_pct(rate: f64) -> String {
    format!("{:.1}", rate * 100.0)
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.to_string()
    } else if limit <= 3 {
        ".".repeat(limit)
    } else {
        let prefix: String = text.chars().take(limit - 3).collect();
        format!("{prefix}...")
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameRecord, Summary, process_frame, truncate};

    #[test]
    fn optional_timings_require_frame_totals_and_unaccounted_saturates() {
        let mut summary = Summary::default();
        let mut stats = Default::default();
        let frame = |total, render, draw| FrameRecord {
            frame: 0,
            frame_total_ns: total,
            render_time_ns: render,
            build_tree_time_ns: None,
            draw_time_ns: draw,
            components: Vec::new(),
        };
        process_frame(frame(None, Some(4), Some(3)), &mut summary, &mut stats);
        process_frame(frame(Some(10), Some(20), Some(3)), &mut summary, &mut stats);
        process_frame(frame(Some(20), Some(5), None), &mut summary, &mut stats);

        assert_eq!(summary.build.count, 3);
        assert_eq!(summary.frame_total.count, 2);
        assert_eq!(summary.render.count, 2);
        assert_eq!(summary.draw.count, 1);
        assert_eq!(summary.build_tree.count, 0);
        assert_eq!(summary.unaccounted.total, 15);
        assert_eq!(
            (summary.unaccounted.min, summary.unaccounted.max),
            (Some(0), Some(15))
        );
    }

    #[test]
    fn truncation_handles_unicode_and_small_limits() {
        assert_eq!(truncate("component", 6), "com...");
        assert_eq!(truncate("ééééééé", 6), "ééé...");
        assert_eq!(truncate("🙂🙂", 2), "🙂🙂");
        assert_eq!(truncate("component", 0), "");
        assert_eq!(truncate("component", 2), "..");
    }
}
