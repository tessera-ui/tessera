use std::{env, process::Command};

pub(crate) fn pixel_near(before: &[u8], after: &[u8]) -> bool {
    use material_color_utilities::utils::color::linearized;

    // Compare rounding in encoded and linear light; atlas coverage adds a step.
    let opaque = before.get(3).is_none_or(|a| *a == 255) && after.get(3).is_none_or(|a| *a == 255);
    let tolerance = if opaque { 1 } else { 2 };
    before
        .iter()
        .zip(after)
        .enumerate()
        .all(|(channel, (a, b))| {
            a.abs_diff(*b) <= tolerance
                || channel < 3
                    && (linearized::<f64>(*a) - linearized::<f64>(*b)).abs() <= 100.0 / 255.0
        })
}

pub(crate) fn assert_pixels_near(before: &[u8], after: &[u8], width: usize) {
    assert_eq!(before.len(), after.len());
    for (index, (before, after)) in before
        .chunks_exact(4)
        .zip(after.chunks_exact(4))
        .enumerate()
    {
        assert!(
            pixel_near(before, after),
            "pixel at ({}, {}): {before:?} != {after:?}",
            index % width,
            index / width
        );
    }
}

pub(crate) fn run_isolated_test(name: &str, test: impl FnOnce()) {
    if env::var("TESSERA_ISOLATED_TEST").as_deref() == Ok(name) {
        test();
        return;
    }

    let output = Command::new(env::current_exe().expect("test executable"))
        .args([
            "--exact",
            name,
            "--include-ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env("TESSERA_ISOLATED_TEST", name)
        .output()
        .expect("isolated test should start");
    assert!(
        output.status.success(),
        "isolated test {name} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed;"),
        "isolated test {name} must execute exactly one test:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
