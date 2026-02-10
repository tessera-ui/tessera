use std::{
    cmp::Reverse,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result, bail};
use comfy_table::{Cell, Color, ContentArrangement, Row, Table, presets::UTF8_FULL};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::output;

#[derive(Deserialize)]
struct FrameRecord {
    #[allow(dead_code)]
    frame: u64,
    render_time_ns: Option<u128>,
    build_tree_time_ns: Option<u128>,
    draw_time_ns: Option<u128>,
    frame_total_ns: Option<u128>,
    layout_diagnostics: Option<LayoutDiagnosticsRecord>,
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

#[derive(Deserialize, Clone, Copy)]
struct LayoutDiagnosticsRecord {
    dirty_nodes_param: u64,
    dirty_nodes_structural: u64,
    dirty_nodes_with_ancestors: u64,
    dirty_expand_ns: u64,
    measure_node_calls: u64,
    cache_hits_direct: u64,
    cache_hits_boundary: u64,
    cache_miss_no_entry: u64,
    cache_miss_constraint: u64,
    cache_miss_dirty_self: u64,
    cache_miss_child_size: u64,
    cache_store_count: u64,
    cache_drop_non_cacheable_count: u64,
}

#[derive(Deserialize)]
struct JsonLineProbe {
    format: Option<String>,
    frame: Option<u64>,
}

#[derive(Default, Clone, Copy)]
struct MetricSummary {
    sum: u128,
    count: u64,
    min: Option<u128>,
    max: Option<u128>,
}

impl MetricSummary {
    fn record(&mut self, value: u128) {
        self.sum += value;
        self.count += 1;
        self.min = Some(self.min.map_or(value, |v| v.min(value)));
        self.max = Some(self.max.map_or(value, |v| v.max(value)));
    }
}

#[derive(Default)]
struct LayoutDiagnosticsSummary {
    frames: u64,
    dirty_nodes_param: MetricSummary,
    dirty_nodes_structural: MetricSummary,
    dirty_nodes_with_ancestors: MetricSummary,
    dirty_expand_ns: MetricSummary,
    measure_node_calls: MetricSummary,
    cache_hits_direct: MetricSummary,
    cache_hits_boundary: MetricSummary,
    cache_miss_no_entry: MetricSummary,
    cache_miss_constraint: MetricSummary,
    cache_miss_dirty_self: MetricSummary,
    cache_miss_child_size: MetricSummary,
    cache_store_count: MetricSummary,
    cache_drop_non_cacheable_count: MetricSummary,
}

impl LayoutDiagnosticsSummary {
    fn record(&mut self, value: LayoutDiagnosticsRecord) {
        self.frames += 1;
        self.dirty_nodes_param
            .record(u128::from(value.dirty_nodes_param));
        self.dirty_nodes_structural
            .record(u128::from(value.dirty_nodes_structural));
        self.dirty_nodes_with_ancestors
            .record(u128::from(value.dirty_nodes_with_ancestors));
        self.dirty_expand_ns
            .record(u128::from(value.dirty_expand_ns));
        self.measure_node_calls
            .record(u128::from(value.measure_node_calls));
        self.cache_hits_direct
            .record(u128::from(value.cache_hits_direct));
        self.cache_hits_boundary
            .record(u128::from(value.cache_hits_boundary));
        self.cache_miss_no_entry
            .record(u128::from(value.cache_miss_no_entry));
        self.cache_miss_constraint
            .record(u128::from(value.cache_miss_constraint));
        self.cache_miss_dirty_self
            .record(u128::from(value.cache_miss_dirty_self));
        self.cache_miss_child_size
            .record(u128::from(value.cache_miss_child_size));
        self.cache_store_count
            .record(u128::from(value.cache_store_count));
        self.cache_drop_non_cacheable_count
            .record(u128::from(value.cache_drop_non_cacheable_count));
    }
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
    build_tree_total: u128,
    build_tree_min: Option<u128>,
    build_tree_max: Option<u128>,
    build_tree_count: u64,
    draw_total: u128,
    draw_min: Option<u128>,
    draw_max: Option<u128>,
    draw_count: u64,
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
    unaccounted_total_sum: u128,
    unaccounted_total_min: Option<u128>,
    unaccounted_total_max: Option<u128>,
    unaccounted_total_count: u64,
    node_count: u64,
    cache_hit: u64,
    cache_miss: u64,
    cache_unknown: u64,
    layout_diagnostics: LayoutDiagnosticsSummary,
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

pub fn analyze(
    path: &Path,
    top: usize,
    min_count: u64,
    skip_invalid: bool,
    csv: Option<&Path>,
) -> Result<()> {
    output::status("Analyzing", path.display().to_string());
    let file = File::open(path)
        .with_context(|| format!("failed to open profiler file at {}", path.display()))?;
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
                if let Ok(probe) = serde_json::from_str::<JsonLineProbe>(trimmed) {
                    if probe.format.as_deref() == Some("tessera-profiler") {
                        continue;
                    }
                    if probe.frame.is_some() {
                        bail!("invalid frame record at line {}", line_idx + 1);
                    }
                    if !skip_invalid {
                        bail!("unrecognized JSON line at {}", line_idx + 1);
                    }
                } else if !skip_invalid {
                    bail!("unrecognized JSON line at {}", line_idx + 1);
                }
            }
        }
    }

    if summary.frames == 0 {
        bail!("no frame records found");
    }

    if let Some(path) = csv {
        export_csv(path, &stats_by_name)?;
        println!("{} {}", "Wrote CSV:".green(), path.display());
    }

    print_summary(&summary);
    print_top_sections(&stats_by_name, top, min_count);
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

fn export_csv(path: &Path, stats: &HashMap<String, Stats>) -> Result<()> {
    let mut rows: Vec<(&String, &Stats)> = stats.iter().collect();
    rows.sort_by_key(|(_, stat)| Reverse(stat.total_ns()));

    let file = File::create(path)
        .with_context(|| format!("failed to create CSV output file at {}", path.display()))?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(file);

    for (name, stat) in rows {
        let count = stat.count;
        let build_avg_ns = if count == 0 {
            0.0
        } else {
            stat.build_ns as f64 / count as f64
        };
        let measure_avg_ns = if count == 0 {
            0.0
        } else {
            stat.measure_ns as f64 / count as f64
        };
        let input_avg_ns = if count == 0 {
            0.0
        } else {
            stat.input_ns as f64 / count as f64
        };
        let total_ns = stat.total_ns();
        let total_avg_ns = if count == 0 {
            0.0
        } else {
            total_ns as f64 / count as f64
        };

        writer
            .serialize(CsvRow {
                component: name.to_string(),
                count,

                build_total_ns: stat.build_ns,
                build_total_ms: stat.build_ns as f64 / 1_000_000.0,
                build_avg_us: build_avg_ns / 1_000.0,

                measure_total_ns: stat.measure_ns,
                measure_total_ms: stat.measure_ns as f64 / 1_000_000.0,
                measure_avg_us: measure_avg_ns / 1_000.0,

                input_total_ns: stat.input_ns,
                input_total_ms: stat.input_ns as f64 / 1_000_000.0,
                input_avg_us: input_avg_ns / 1_000.0,

                total_total_ns: total_ns,
                total_total_ms: total_ns as f64 / 1_000_000.0,
                total_avg_us: total_avg_ns / 1_000.0,

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

    if let Some(layout_diagnostics) = frame.layout_diagnostics {
        summary.layout_diagnostics.record(layout_diagnostics);
    }

    if let Some(total) = frame.frame_total_ns {
        summary.frame_total_sum += total;
        summary.frame_total_count += 1;
        summary.frame_total_min = Some(summary.frame_total_min.map_or(total, |v| v.min(total)));
        summary.frame_total_max = Some(summary.frame_total_max.map_or(total, |v| v.max(total)));

        if let Some(build_tree) = frame.build_tree_time_ns {
            summary.build_tree_total += build_tree;
            summary.build_tree_count += 1;
            summary.build_tree_min = Some(
                summary
                    .build_tree_min
                    .map_or(build_tree, |v| v.min(build_tree)),
            );
            summary.build_tree_max = Some(
                summary
                    .build_tree_max
                    .map_or(build_tree, |v| v.max(build_tree)),
            );
        }
        if let Some(draw) = frame.draw_time_ns {
            summary.draw_total += draw;
            summary.draw_count += 1;
            summary.draw_min = Some(summary.draw_min.map_or(draw, |v| v.min(draw)));
            summary.draw_max = Some(summary.draw_max.map_or(draw, |v| v.max(draw)));
        }

        if let Some(render) = frame.render_time_ns {
            summary.render_total += render;
            summary.render_count += 1;

            let accounted = frame_totals.build + frame_totals.measure + frame_totals.input + render;
            let unaccounted = total.saturating_sub(accounted);
            summary.unaccounted_total_sum += unaccounted;
            summary.unaccounted_total_count += 1;
            summary.unaccounted_total_min = Some(
                summary
                    .unaccounted_total_min
                    .map_or(unaccounted, |v| v.min(unaccounted)),
            );
            summary.unaccounted_total_max = Some(
                summary
                    .unaccounted_total_max
                    .map_or(unaccounted, |v| v.max(unaccounted)),
            );
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

    if summary.frame_total_count > 0 {
        let avg = summary.frame_total_sum as f64 / summary.frame_total_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Frame total (wall)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.frame_total_min.unwrap_or(0) as f64)
            )),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.frame_total_max.unwrap_or(0) as f64)
            )),
        ]));
    }

    if summary.build_tree_count > 0 {
        let avg = summary.build_tree_total as f64 / summary.build_tree_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Build tree (wall)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.build_tree_min.unwrap_or(0) as f64)
            )),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.build_tree_max.unwrap_or(0) as f64)
            )),
        ]));
    }
    if summary.draw_count > 0 {
        let avg = summary.draw_total as f64 / summary.draw_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Draw/compute (wall)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.draw_min.unwrap_or(0) as f64)
            )),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.draw_max.unwrap_or(0) as f64)
            )),
        ]));
    }
    if summary.render_count > 0 {
        let avg = summary.render_total as f64 / summary.render_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Render (wall)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(""),
            Cell::new(""),
        ]));
    }

    if summary.build_total_count > 0 {
        let avg = summary.build_total_sum as f64 / summary.build_total_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Build total (exclusive CPU)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.build_total_min.unwrap_or(0) as f64)
            )),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.build_total_max.unwrap_or(0) as f64)
            )),
        ]));
    }
    if summary.measure_total_count > 0 {
        let avg = summary.measure_total_sum as f64 / summary.measure_total_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Measure total (exclusive CPU)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.measure_total_min.unwrap_or(0) as f64)
            )),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.measure_total_max.unwrap_or(0) as f64)
            )),
        ]));
    }
    if summary.input_total_count > 0 {
        let avg = summary.input_total_sum as f64 / summary.input_total_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Input total (exclusive CPU)"),
            Cell::new(format!("{} ms", format_ms(avg))),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.input_total_min.unwrap_or(0) as f64)
            )),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.input_total_max.unwrap_or(0) as f64)
            )),
        ]));
    }

    if summary.unaccounted_total_count > 0 {
        let avg = summary.unaccounted_total_sum as f64 / summary.unaccounted_total_count as f64;
        table.add_row(Row::from(vec![
            Cell::new("Unaccounted (wall)").fg(Color::DarkGrey),
            Cell::new(format!("{} ms", format_ms(avg))).fg(Color::DarkGrey),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.unaccounted_total_min.unwrap_or(0) as f64)
            ))
            .fg(Color::DarkGrey),
            Cell::new(format!(
                "{} ms",
                format_ms(summary.unaccounted_total_max.unwrap_or(0) as f64)
            ))
            .fg(Color::DarkGrey),
        ]));
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

    if summary.layout_diagnostics.frames > 0 {
        let diag = &summary.layout_diagnostics;
        add_metric_count_row(
            &mut table,
            "Layout dirty nodes (params)",
            &diag.dirty_nodes_param,
        );
        add_metric_count_row(
            &mut table,
            "Layout dirty nodes (structural)",
            &diag.dirty_nodes_structural,
        );
        add_metric_count_row(
            &mut table,
            "Layout dirty nodes (effective)",
            &diag.dirty_nodes_with_ancestors,
        );
        add_metric_ns_row(
            &mut table,
            "Layout dirty prepare (CPU)",
            &diag.dirty_expand_ns,
        );
        add_metric_count_row(&mut table, "Layout measure calls", &diag.measure_node_calls);
        add_metric_count_row(&mut table, "Layout hit: direct", &diag.cache_hits_direct);
        add_metric_count_row(
            &mut table,
            "Layout hit: boundary",
            &diag.cache_hits_boundary,
        );
        add_metric_count_row(
            &mut table,
            "Layout miss: no entry",
            &diag.cache_miss_no_entry,
        );
        add_metric_count_row(
            &mut table,
            "Layout miss: constraint",
            &diag.cache_miss_constraint,
        );
        add_metric_count_row(
            &mut table,
            "Layout miss: dirty self",
            &diag.cache_miss_dirty_self,
        );
        add_metric_count_row(
            &mut table,
            "Layout miss: child size",
            &diag.cache_miss_child_size,
        );
        add_metric_count_row(&mut table, "Layout cache stores", &diag.cache_store_count);
        add_metric_count_row(
            &mut table,
            "Layout cache drops",
            &diag.cache_drop_non_cacheable_count,
        );

        let total_hits = diag.cache_hits_direct.sum + diag.cache_hits_boundary.sum;
        let total_misses = diag.cache_miss_no_entry.sum
            + diag.cache_miss_constraint.sum
            + diag.cache_miss_dirty_self.sum
            + diag.cache_miss_child_size.sum;
        let total_attempts = total_hits + total_misses;
        if total_attempts > 0 {
            let rate = total_hits as f64 / total_attempts as f64;
            table.add_row(Row::from(vec![
                Cell::new("Layout cache hit rate (diag)"),
                Cell::new(format!(
                    "{}% (hit {}, miss {})",
                    format_pct(rate),
                    total_hits,
                    total_misses
                )),
                Cell::new(""),
                Cell::new(""),
            ]));
        }

        if total_hits > 0 {
            let boundary_rate = diag.cache_hits_boundary.sum as f64 / total_hits as f64;
            table.add_row(Row::from(vec![
                Cell::new("Layout boundary hit ratio"),
                Cell::new(format!(
                    "{}% (boundary {} / total hits {})",
                    format_pct(boundary_rate),
                    diag.cache_hits_boundary.sum,
                    total_hits
                )),
                Cell::new(""),
                Cell::new(""),
            ]));
        }
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

    rows.sort_by_key(|(_, stat)| Reverse(stat.measure_ns));
    print_section("Top by measure_ns (exclusive)", &rows, top, |s| {
        s.measure_ns
    });

    rows.sort_by_key(|(_, stat)| Reverse(stat.build_ns));
    print_section("Top by build_ns (exclusive)", &rows, top, |s| s.build_ns);

    rows.sort_by_key(|(_, stat)| Reverse(stat.input_ns));
    print_section("Top by input_ns (exclusive)", &rows, top, |s| s.input_ns);

    rows.sort_by_key(|(_, stat)| Reverse(stat.total_ns()));
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
        let avg_ns = if stat.count == 0 {
            0.0
        } else {
            total_ns as f64 / stat.count as f64
        };
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

fn add_metric_count_row(table: &mut Table, label: &str, metric: &MetricSummary) {
    if metric.count == 0 {
        return;
    }
    let avg = metric.sum as f64 / metric.count as f64;
    table.add_row(Row::from(vec![
        Cell::new(label),
        Cell::new(format!("{avg:.2}")),
        Cell::new(metric.min.unwrap_or(0).to_string()),
        Cell::new(metric.max.unwrap_or(0).to_string()),
    ]));
}

fn add_metric_ns_row(table: &mut Table, label: &str, metric: &MetricSummary) {
    if metric.count == 0 {
        return;
    }
    let avg_ns = metric.sum as f64 / metric.count as f64;
    table.add_row(Row::from(vec![
        Cell::new(label),
        Cell::new(format!("{} us", format_us(avg_ns))),
        Cell::new(format!("{} us", format_us(metric.min.unwrap_or(0) as f64))),
        Cell::new(format!("{} us", format_us(metric.max.unwrap_or(0) as f64))),
    ]));
}
