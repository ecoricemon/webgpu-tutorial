use eg_render::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use winit::event::*;

pub struct App {
    input_state: Rc<RefCell<InputState>>,
    render_state: Rc<RefCell<RenderState>>,
}

impl App {
    pub async fn new() -> Self {
        // Input state
        let input_state = Rc::new(RefCell::new(InputState::new()));
        // Render state & winit's event loop
        let (render_state, event_loop) = RenderState::new().await;
        // Connect the event loop to the input state
        RenderState::set_event_handlers(
            event_loop,
            render_state.winit_window.id(),
            input_state.clone(),
        );
        // Wrap the render state with Rc<RefCell<>>
        let render_state = Rc::new(RefCell::new(render_state));

        Self {
            input_state,
            render_state,
        }
    }

    pub fn run(&self) {
        let animate = Rc::new(RefCell::new(None));
        let animate_drop = animate.clone();
        let input_state = self.input_state.clone();
        let render_state = self.render_state.clone();
        *animate_drop.borrow_mut() = Some(Closure::<dyn FnMut(f32)>::new(move |time: f32| {
            let mut input_state = input_state.borrow_mut();
            let mut render_state = render_state.borrow_mut();

            // Input
            input_state.consume().for_each(|event| match event {
                WindowEvent::Resized(_) => {
                    // Don't use physical_size in Resized for now because it doesn't work as expected in Chrome device mode.
                    let (new_width, new_height) = render_state.get_scaled_size();
                    render_state.resize(new_width, new_height);
                }
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                } => {
                    render_state.mousemove(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                } if state == ElementState::Pressed && button == MouseButton::Left => {
                    render_state.click();
                }
                WindowEvent::Touch(Touch {
                    device_id: _,
                    phase,
                    location,
                    force: _,
                    id: _,
                }) if phase == TouchPhase::Started => {
                    render_state.mousemove(location.x as f32, location.y as f32);
                }
                _ => (),
            });
            // Render
            render_state.render(time);
            // Request next frame
            render_state.request_animation_frame(animate.borrow().as_ref().unwrap());
        }));

        self.render_state
            .borrow()
            .request_animation_frame(animate_drop.borrow().as_ref().unwrap());
    }

    pub fn test_camera(&mut self, camera: (f32, f32, f32), at: (f32, f32, f32)) {
        self.render_state
            .borrow_mut()
            .set_camera(Some(camera), Some(at), None);
    }
}
