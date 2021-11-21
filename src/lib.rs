use std::cell::RefCell;
use std::rc::Rc;

extern crate console_error_panic_hook;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

mod pipeline;
use pipeline::{AppContext, Stage};

mod stages;
use stages::RenderStage;

mod web_util;
use web_util::{
    body,
    canvas,
    perf_to_system,
    performance,
    request_animation_frame,
    set_canvas_dimensions,
};


#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let gl = Rc::new(
        canvas()
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?
    );

    let ctx = Rc::new(
        AppContext {
            width: body().client_width() as u32,
            height: body().client_height() as u32,
            boot_time: perf_to_system(performance().now()),
        }
    );

    set_canvas_dimensions(ctx.width, ctx.height);

    let mut compute_stage = RenderStage::new(
        gl,
        ctx,
    );

    event_loop(move || {
        compute_stage.render();
    });

    Ok(())
}

fn event_loop<F: FnMut() + 'static>(mut on_frame: F) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        on_frame();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}
