#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use eframe::WebRunner;
use js_sys::Uint8Array;
use ussal_shared::BenchArchive;
use ussal_viewer::App;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlElement, Request, RequestInit, RequestMode, Response};

pub fn main() {
    wasm_bindgen_futures::spawn_local(run());
}

async fn run() {
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

    let raw_cbor = get_bench_history().await;
    let archive = BenchArchive::load_from_cbor(&raw_cbor);

    let runner = WebRunner::new();
    runner
        .start(
            "the-id",
            eframe::WebOptions::default(),
            Box::new(|cc| Box::new(crate::App::new(cc, archive))),
        )
        .await
        .unwrap();
}

async fn get_bench_history() -> Vec<u8> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("bench_history.cbor", &opts).unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let js_value = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
    Uint8Array::new(&js_value).to_vec()
}
