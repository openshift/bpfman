/// Example on how to use the library to build rustdoc JSON. Run it like this:
/// ```bash
/// cargo run --example build-rustdoc-json path/to/Cargo.toml
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build it
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path(std::env::args().nth(1).unwrap())
        .build()?;
    println!("Built and wrote rustdoc JSON to {:?}", &json_path);

    // Show it
    show_json(&json_path)?;

    Ok(())
}

/// A simple hack to conveniently show the JSON output
fn show_json(path: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c");
    cmd.arg(&format!("cat {path:?} | python3 -m json.tool | less"));
    cmd.spawn()?.wait()
}
