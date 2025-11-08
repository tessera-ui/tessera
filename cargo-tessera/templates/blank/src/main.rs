#[cfg(not(target_os = "android"))]
fn main() {
    {{project_name_snake}}::desktop_main();
}

#[cfg(target_os = "android")]
fn main() {
    // Android uses `android_main` defined in src/lib.rs
}
