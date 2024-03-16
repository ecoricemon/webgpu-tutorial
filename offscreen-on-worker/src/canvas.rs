use std::{ops::Deref, ptr::NonNull};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug, Clone)]
pub struct Canvas {
    element: web_sys::HtmlCanvasElement,
    handle: u32,
}

impl Canvas {
    pub fn new(selectors: &str, handle: u32) -> Self {
        // 0 is reserved for window itself.
        assert!(handle > 0);

        // Injects `data-raw-handle` attribute into the canvas element.
        // This is required by `wgpu::Surface` and `raw-window-handle`.
        let element = Self::get_canvas_element(selectors);
        element
            .set_attribute("data-raw-handle", handle.to_string().as_str())
            .unwrap();

        Self { element, handle }
    }

    pub fn get_canvas_element(selectors: &str) -> web_sys::HtmlCanvasElement {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document.query_selector(selectors).unwrap().unwrap();
        let canvas = element.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let scale_factor = window.device_pixel_ratio();
        let width = (canvas.client_width() as f64 * scale_factor) as u32;
        let height = (canvas.client_height() as f64 * scale_factor) as u32;
        canvas.set_width(width);
        canvas.set_height(height);
        canvas
    }

    #[inline]
    pub fn handle(&self) -> u32 {
        self.handle
    }
}

impl Deref for Canvas {
    type Target = web_sys::HtmlCanvasElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl raw_window_handle::HasWindowHandle for Canvas {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{RawWindowHandle, WebCanvasWindowHandle, WindowHandle};

        let value: &JsValue = &self.element;
        let obj: NonNull<std::ffi::c_void> = NonNull::from(value).cast();
        let handle = WebCanvasWindowHandle::new(obj);
        let raw = RawWindowHandle::WebCanvas(handle);
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

impl raw_window_handle::HasDisplayHandle for Canvas {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{DisplayHandle, RawDisplayHandle, WebDisplayHandle};
        let handle = WebDisplayHandle::new();
        let raw = RawDisplayHandle::Web(handle);
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}

#[derive(Debug, Clone)]
pub struct OffscreenCanvas {
    inner: web_sys::OffscreenCanvas,
    handle: u32,
}

impl OffscreenCanvas {
    pub const fn new(canvas: web_sys::OffscreenCanvas, handle: u32) -> Self {
        Self {
            inner: canvas,
            handle,
        }
    }

    pub fn each(self) -> (web_sys::OffscreenCanvas, u32) {
        (self.inner, self.handle)
    }
}

impl Deref for OffscreenCanvas {
    type Target = web_sys::OffscreenCanvas;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<&Canvas> for OffscreenCanvas {
    fn from(value: &Canvas) -> Self {
        let offscreen = value.element.transfer_control_to_offscreen().unwrap();
        let handle = value.handle;
        Self::new(offscreen, handle)
    }
}

impl raw_window_handle::HasWindowHandle for OffscreenCanvas {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{RawWindowHandle, WebOffscreenCanvasWindowHandle, WindowHandle};

        let value: &JsValue = &self.inner;
        let obj: NonNull<std::ffi::c_void> = NonNull::from(value).cast();
        let handle = WebOffscreenCanvasWindowHandle::new(obj);
        let raw = RawWindowHandle::WebOffscreenCanvas(handle);
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

impl raw_window_handle::HasDisplayHandle for OffscreenCanvas {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{DisplayHandle, RawDisplayHandle, WebDisplayHandle};
        let handle = WebDisplayHandle::new();
        let raw = RawDisplayHandle::Web(handle);
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}
