//! An example of how to render water using WebGL + Rust + WebAssembly.
//!
//! We'll try to heavily over comment the code so that it's more accessible to those that
//! are less familiar with the techniques that are used.
//!
//! In a real application you'd split things up into different modules and files,
//! but I tend to prefer tutorials that are all in one file that you can scroll up and down in
//! and soak up what you see vs. needing to hop around different files.
//!
//! If you have any questions or comments feel free to open an issue on GitHub!
//!
//! https://github.com/chinedufn/webgl-water-tutorial
//!
//! Heavily inspired by this @thinmatrix tutorial:
//!   - https://www.youtube.com/watch?v=HusvGeEDU_U&list=PLRIWtICgwaX23jiqVByUs0bqhnalNTNZh

#![deny(missing_docs)]

extern crate wasm_bindgen;
pub(crate) use self::app::*;
use self::canvas::*;
use self::controls::*;
use self::render::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::*;

mod app;
mod canvas;
mod controls;
mod generate_sphere;
mod render;
mod shader;
mod simulation;
mod webgl_object;

/// Used to run the application from the web
#[wasm_bindgen]
pub struct WebClient {
    app: Rc<App>,
    gl: Rc<WebGlRenderingContext>,
    renderer: WebRenderer,
}
#[wasm_bindgen]
impl WebClient {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebClient {
        console_error_panic_hook::set_once();

        let app = Rc::new(App::new());

        let gl = Rc::new(create_webgl_context(Rc::clone(&app)).unwrap());
        append_controls(Rc::clone(&app)).expect("Append controls");
        append_values(Rc::clone(&app)).expect("Append values");

        let renderer = WebRenderer::new(&gl);

        WebClient { app, gl, renderer }
    }

    /// Start our WebGL Water application. `index.html` will call this function in order
    /// to begin rendering.
    pub fn start(&self) -> Result<(), JsValue> {
        let gl = &self.gl;

        Ok(())
    }

    /// Update our simulation
    pub fn update(&self, dt: f32) {
        self.app.store.borrow_mut().msg(&Msg::AdvanceClock(dt));
        self.app.store.borrow_mut().msg(&Msg::UpdateSimulation(dt));
        let l = self.app.store.borrow().state.simulation.get_total_angular_momentum();
        let p = self.app.store.borrow().state.simulation.get_total_momentum();
        let e = self.app.store.borrow().state.simulation.get_total_energy();

        update_values(self.app.clone(), l, p, e);

        // web_sys::console::log_1(&format!("L: {:?}, P: {:?}, E: {:?}", l, p, e).into());
    }

    /// Render the scene. `index.html` will call this once every requestAnimationFrame
    pub fn render(&mut self) {
        self.renderer
            .render(&self.gl, &self.app.store.borrow().state, self.app.assets());
    }
}

impl Default for WebClient {
    fn default() -> Self {
        Self::new()
    }
}
