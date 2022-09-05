#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1).cloned();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "bench viewer",
        native_options,
        Box::new(|cc| Box::new(crate::TemplateApp::new(cc, rom_path))),
    );
}

#[cfg(target_arch = "wasm32")]
pub fn main() -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.

    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;
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

    eframe::start_web(
        "the-id",
        eframe::WebOptions::default(),
        Box::new(|cc| Box::new(crate::TemplateApp::new(cc, None))),
    )
    .map(|_| ())
}
