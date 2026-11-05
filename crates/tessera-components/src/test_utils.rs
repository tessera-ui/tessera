use std::{env, process::Command};

pub(crate) fn assert_restored_pixels(before: &[u8], after: &[u8], width: usize) {
    assert_eq!(before.len(), after.len());
    for (index, (before, after)) in before
        .chunks_exact(4)
        .zip(after.chunks_exact(4))
        .enumerate()
    {
        // Shadow atlas resampling adds a rounding step to translucent coverage.
        let tolerance = if before[3] == 255 && after[3] == 255 {
            1
        } else {
            2
        };
        assert!(
            before
                .iter()
                .zip(after)
                .all(|(a, b)| a.abs_diff(*b) <= tolerance),
            "restored pixel at ({}, {}): {before:?} != {after:?}",
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
