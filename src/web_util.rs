use std::time::{Duration, SystemTime, UNIX_EPOCH};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;


pub fn window() -> web_sys::Window {
    return web_sys::window().expect("should have a window in this context");
}

pub fn document() -> web_sys::Document {
    return window().document().expect("should have a document in this context");
}

pub fn canvas() -> web_sys::HtmlCanvasElement {
    return document()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("should have a canvas in this context");
}

pub fn body() -> web_sys::HtmlBodyElement {
    return document()
        .body()
        .unwrap()
        .dyn_into::<web_sys::HtmlBodyElement>()
        .expect("should have a body in this context");
}

pub fn performance() -> web_sys::Performance {
    return window().performance().expect("should have a performance in this context");
}

pub fn set_canvas_dimensions(width: u32, height: u32) {
    canvas().set_width(width);
    canvas().set_height(height);
}

pub fn perf_to_system(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().expect("no global `window` exists")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
