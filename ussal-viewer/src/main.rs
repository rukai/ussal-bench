#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::App;

use ussal_shared::BenchRun;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bench_run_path = args.get(1).map(|x| x.as_ref()).unwrap_or("name");
    let bench_run = BenchRun::load(bench_run_path);
    eframe::run_native(
        "bench viewer",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(crate::App::new(cc, bench_run))),
    );
}

#[cfg(target_arch = "wasm32")]
pub fn main() -> Result<(), eframe::wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let document = web_sys::window().unwrap().document().unwrap();

    let body = document.body().unwrap();

    // TODO: Maybe we could upstream this boilerplate into eframe
    let canvas = document.create_element("canvas").unwrap();
    canvas.set_id("the-id");
    body.append_child(&canvas)
        .expect("Append canvas to HTML body");
    body.style()
        .set_css_text("margin: 0; height: 100%; width: 100%");
    document
        .document_element()
        .unwrap()
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .style()
        .set_css_text("margin: 0; height: 100%; width: 100%");

    let raw_cbor = include_bytes!("../../name.cbor");
    let bench_run = BenchRun::load_from_cbor(raw_cbor);

    eframe::start_web(
        "the-id",
        eframe::WebOptions::default(),
        Box::new(|cc| Box::new(crate::App::new(cc, bench_run))),
    )
    .map(|_| ())
}
