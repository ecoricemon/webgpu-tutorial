use std::borrow::BorrowMut;

use eg_app::App;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    {
        // Show panic messages on browsers
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_error_panic_hook::set_once();
    }
}

#[wasm_bindgen]
pub struct MyApp {
    app: Option<App>,
}

#[wasm_bindgen]
impl MyApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { app: None }
    }

    pub async fn run(&mut self) {
        let app = App::new().await;
        app.run();
        self.app = Some(app);
    }

    pub fn set_camera(
        &mut self,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32,
        at_x: f32,
        at_y: f32,
        at_z: f32,
    ) {
        if let Some(app) = self.app.borrow_mut() {
            app.test_camera((camera_x, camera_y, camera_z), (at_x, at_y, at_z));
        }
    }
}
