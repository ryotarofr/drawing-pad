use drawing_pad::DrawingApp;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = DrawingApp::new();
    event_loop.run_app(&mut app).unwrap();
}