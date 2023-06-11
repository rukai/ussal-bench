use log::error;
use subprocess::{Exec, Redirection};

pub fn generate_web() {
    std::fs::create_dir_all("bench_ci_web_root").unwrap();
    std::fs::write("bench_ci_web_root/index.html", include_bytes!("index.html")).unwrap();

    // TODO: in production builds we should fetch the wasm from a github release instead of building locally.
    //       It will be: way faster, less non-cargo dependencies and avoid wasm-bindgen version mismatches.
    //
    //       For now this works fine though and has the advantage of being easier to test / more reproducible

    run_command_in_dir("cargo", &["build", "--release"], "ussal-viewer-web");

    let wasm_path = "ussal-viewer-web/target/wasm32-unknown-unknown/release/ussal-viewer-web.wasm";
    let destination_dir = "ussal-viewer-web/target/generated";
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen
        .web(true)
        .unwrap()
        .omit_default_module_path(false)
        .input_path(wasm_path)
        .generate(destination_dir)
        .unwrap();

    let wasm_file_name = "ussal-viewer-web_bg.wasm";
    if Exec::cmd("wasm-opt").args(&["--help"]).capture().is_ok() {
        run_command_in_dir(
            "wasm-opt",
            &["-Oz", "-o", wasm_file_name, wasm_file_name],
            "ussal-viewer-web/target/generated/",
        );
    } else {
        error!("Skipping wasm-opt because not installed")
    }
    std::fs::copy(
        "ussal-viewer-web/target/generated/ussal-viewer-web_bg.wasm",
        "bench_ci_web_root/ussal-viewer-web_bg.wasm",
    )
    .unwrap();
    std::fs::copy(
        "ussal-viewer-web/target/generated/ussal-viewer-web.js",
        "bench_ci_web_root/ussal-viewer-web.js",
    )
    .unwrap();
}

fn run_command_in_dir(command: &str, args: &[&str], dir: &str) {
    let data = Exec::cmd(command)
        .args(args)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .cwd(dir)
        .capture()
        .unwrap_or_else(|e| panic!("Failed to run the command {command} {args:?}\n{e}"));

    if !data.exit_status.success() {
        panic!(
            "command {} {:?} exited with {:?} and output:\n{}",
            command,
            args,
            data.exit_status,
            data.stdout_str()
        )
    }
}
