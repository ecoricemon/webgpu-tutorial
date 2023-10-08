use std::collections::vec_deque::Drain;
use std::collections::VecDeque;
use winit::event::WindowEvent;

pub struct InputState {
    queue: VecDeque<WindowEvent>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    #[inline]
    pub fn produce(&mut self, event: WindowEvent) {
        self.queue.push_back(event);
    }

    #[inline]
    pub fn consume(&mut self) -> Drain<'_, WindowEvent> {
        self.queue.drain(..)
    }
}
