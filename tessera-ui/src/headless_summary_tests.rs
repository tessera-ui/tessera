//! Compact headless profiling through the actual worker protocol.
//!
//! ## Usage
//!
//! Keep performance captures small while retaining explicit scene inspection.

use std::{fs, process::Command};

use super::*;

#[test]
#[ignore = "requires a native GPU adapter; spawns an isolated profiling process"]
fn worker_summary_retains_real_timings_counters_and_explicit_snapshot() {
    const CHILD: &str = "TESSERA_TEST_HEADLESS_SUMMARY_OUTPUT";
    let Some(output) = std::env::var_os(CHILD) else {
        let path = std::env::temp_dir().join(format!(
            "tessera-headless-summary-{}.jsonl",
            std::process::id()
        ));
        let result = Command::new(std::env::current_exe().expect("Test executable"))
            .args([
                "--exact",
                "headless::protocol_tests::summary_tests::worker_summary_retains_real_timings_counters_and_explicit_snapshot",
                "--ignored",
                "--nocapture",
            ])
            .env(CHILD, &path)
            .env("TESSERA_HEADLESS_GPU_TIMING", "1")
            .output()
            .expect("Isolated profiling process");
        assert!(
            result.status.success(),
            "child stdout:\n{}\nchild stderr:\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            String::from_utf8_lossy(&result.stdout).contains("test result: ok. 1 passed;"),
            "Exact child test must execute once"
        );
        let records: Vec<Value> = fs::read_to_string(&path)
            .expect("Actual profiler file")
            .lines()
            .map(|line| serde_json::from_str(line).expect("Profiler JSON"))
            .filter(|record: &Value| record["type"] == "frame")
            .collect();
        fs::remove_file(path).expect("Remove isolated trace");
        assert_eq!(records.len(), 4);
        for (index, record) in records.iter().enumerate() {
            assert_eq!(record["frame"], index as u64);
            for timing in [
                "frame_total_ns",
                "build_tree_time_ns",
                "draw_time_ns",
                "record_time_ns",
                "render_time_ns",
            ] {
                assert!(record[timing].is_u64(), "Missing actual {timing}");
            }
            assert!(record["frame_total_ns"].as_u64().unwrap() > 0);
            assert!(
                record["layout_diagnostics"]["measure_node_calls"]
                    .as_u64()
                    .unwrap()
                    >= 1
            );
        }
        for record in &records[..2] {
            assert!(record["components"].as_array().unwrap().is_empty());
            assert!(serde_json::to_vec(record).unwrap().len() < 8192);
        }
        assert!(
            records[0]["layout_diagnostics"]["geometry_nodes_updated"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(records[1]["layout_diagnostics"]["measure_node_calls"], 1);
        assert_eq!(records[1]["layout_diagnostics"]["cache_hits_direct"], 1);
        assert_eq!(records[1]["layout_diagnostics"]["cache_store_count"], 0);
        assert_eq!(
            records[1]["layout_diagnostics"]["geometry_nodes_updated"],
            0
        );
        assert!(!records[2]["components"].as_array().unwrap().is_empty());
        assert!(!records[3]["components"].as_array().unwrap().is_empty());
        if records[2].get("render_gpu_ns").is_some() {
            for record in &records[..2] {
                assert!(record["render_gpu_ns"].as_u64().unwrap() > 0);
            }
        }
        return;
    };
    crate::profiler::set_output_path(output);
    LOCAL_PRESS.set(None);
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            protocol_fixture();
        }),
        Vec::new(),
        HeadlessConfig {
            width: 300,
            height: 200,
            ..HeadlessConfig::default()
        },
    )
    .expect("Native GPU adapter");
    assert!(
        !renderer.profile_summary,
        "Default profiling must retain full detail"
    );
    renderer.profile_summary = true;
    let (_, response) = handle_request(
        &mut renderer,
        WorkerRequest::Render {
            id: Some(1),
            frames: Some(2),
            duration_ms: None,
            out: None,
        },
    )
    .expect("Summary frames through worker protocol");
    assert_eq!(response["frame_index"], 2);
    let (_, response) = handle_request(&mut renderer, WorkerRequest::Snapshot { id: Some(2) })
        .expect("Explicit inspection remains available");
    let bounds = probe_bounds(&response["snapshot"]["root"]).expect("Actual complete scene");
    assert_eq!(bounds["x"], 10.25);
    assert_eq!(bounds["y"], 20.5);
    assert_eq!(bounds["width"], 150.0);
    renderer.profile_summary = false;
    let request: WorkerRequest = serde_json::from_value(json!({
        "cmd":"input", "kind":"press", "x":40.25, "y":35.5
    }))
    .unwrap();
    handle_request(&mut renderer, request).expect("Real input after summary");
    handle_request(
        &mut renderer,
        WorkerRequest::Render {
            id: Some(3),
            frames: Some(2),
            duration_ms: None,
            out: None,
        },
    )
    .expect("Full frames after summary");
    assert_eq!(LOCAL_PRESS.get(), Some(PxPosition::from_f32_xy(48.0, 24.0)));
    crate::profiler::flush_for_test();
}
