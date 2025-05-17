#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use tools_for_210::app::EMULATOR;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use std::time::{Duration, Instant};

    use eframe::UserEvent;
    use tools_for_210::app::LAST_PAINT_ID;
    use winit::{
        event_loop::{ControlFlow, EventLoop},
        platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
    };

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    let mut event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = eframe::create_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(tools_for_210::TemplateApp::new(cc)))),
        &event_loop,
    );

    let event_proxy = event_loop.create_proxy();

    while matches!(
        event_loop.pump_app_events(Some(Duration::from_millis(15)), &mut app),
        PumpStatus::Continue
    ) {
        if update() {
            // if the emulator state has chaned, repaint app
            event_proxy
                .send_event(UserEvent::RequestRepaint {
                    viewport_id: egui::ViewportId::ROOT,
                    when: Instant::now(),
                    cumulative_pass_nr: *LAST_PAINT_ID.lock().unwrap(),
                })
                .unwrap();
        }
    }

    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use std::cell::RefCell;
    use std::rc::Rc;
    use tracing_subscriber::fmt::format::Pretty;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;
    use tracing_web::{performance_layer, MakeWebConsoleWriter};
    use wasm_bindgen::{prelude::Closure, JsCast};

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(EnvFilter::new("debug"))
        .with(perf_layer)
        .with(fmt_layer)
        .init();

    let web_options = eframe::WebOptions {
        ..Default::default()
    };

    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("No window found - critical error");

        let document = window
            .document()
            .expect("No document found - critical error");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id - ensure HTML is properly configured")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement - check element type");
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(tools_for_210::TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }

        // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
        // number of times. After it's done we want all our resources cleaned up. To
        // achieve this we're using an `Rc`. The `Rc` will eventually store the
        // closure we want to execute on each frame, but to start out it contains
        // `None`.
        //
        // After the `Rc` is made we'll actually create the closure, and the closure
        // will reference one of the `Rc` instances. The other `Rc` reference is
        // used to store the closure, request the first frame, and then is dropped
        // by this function.
        //
        // Inside the closure we've got a persistent `Rc` reference, which we use
        // for all future iterations of the loop
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            update();

            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    });
}

#[cfg(target_arch = "wasm32")]
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    use wasm_bindgen::JsCast;
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn update() -> bool {
    let mut emulator = EMULATOR.lock().unwrap();
    emulator.update()
}
