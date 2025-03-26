#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
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
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(tools_for_210::TemplateApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use tracing_subscriber::fmt::format::Pretty;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;
    use tracing_web::{performance_layer, MakeWebConsoleWriter};
    use web_sys::wasm_bindgen::JsCast;

    // Set up comprehensive tracing for web
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

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

    tracing::info!("WEB APPLICATION STARTING - ALL THE LOGS ACTIVATED");
    tracing::debug!("Debug logging enabled");
    tracing::trace!("Trace logging enabled - maximum verbosity");

    // eframe::WebLogger::init(log::LevelFilter::Trace).expect("Failed to initialize WebLogger");

    let web_options = eframe::WebOptions {
        ..Default::default()
    };

    tracing::info!("Spawning web application with ALL THE LOGS");

    wasm_bindgen_futures::spawn_local(async {
        tracing::debug!("Inside async web initialization block");

        let window = web_sys::window().expect("No window found - critical error");
        tracing::trace!("Window object acquired");

        let document = window
            .document()
            .expect("No document found - critical error");
        tracing::trace!("Document object acquired");

        tracing::debug!("Attempting to locate canvas element 'the_canvas_id'");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id - ensure HTML is properly configured")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement - check element type");
        tracing::info!("Canvas element successfully acquired");

        tracing::debug!("Creating web runner and starting application");
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    tracing::info!("Creation context received, initializing application");
                    Ok(Box::new(tools_for_210::TemplateApp::new(cc)))
                }),
            )
            .await;
        tracing::debug!(
            "Application start completed with result: {:?}",
            start_result
        );

        // Remove the loading text and spinner:
        tracing::trace!("Looking for loading_text element to remove");
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    tracing::info!("Application started successfully, removing loading text");
                    loading_text.remove();
                }
                Err(e) => {
                    tracing::error!("APPLICATION CRASHED: {e:?}");
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        } else {
            tracing::warn!("No loading_text element found to remove");
        }

        tracing::info!("Web application initialization complete. ALL THE LOGS ARE FLOWING!");
    });
}
