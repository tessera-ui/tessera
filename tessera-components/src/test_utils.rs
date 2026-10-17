use std::{env, process::Command};

pub(crate) fn run_isolated_test(name: &str, test: impl FnOnce()) {
    if env::var("TESSERA_ISOLATED_TEST").as_deref() == Ok(name) {
        test();
        return;
    }

    let output = Command::new(env::current_exe().expect("test executable"))
        .args(["--exact", name, "--test-threads=1", "--nocapture"])
        .env("TESSERA_ISOLATED_TEST", name)
        .output()
        .expect("isolated test should start");
    assert!(
        output.status.success(),
        "isolated test {name} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
