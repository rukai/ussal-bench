#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use ussal_archive::BenchArchive;
use ussal_viewer::App;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bench_run_path = args.get(1).map(|x| x.as_ref()).unwrap_or("bench.cbor");
    let bench_run = BenchArchive::load(bench_run_path).unwrap();
    eframe::run_native(
        "bench viewer",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(crate::App::new(cc, bench_run))),
    )
    .unwrap();
}
