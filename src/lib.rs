pub mod graphics;
pub mod sounds;
pub mod window;

use std::sync::atomic::{AtomicBool, Ordering};

use window::{App, Graphics};
use winit::event_loop::EventLoop;

static STARTED: AtomicBool = AtomicBool::new(false);

pub fn run() {
    if STARTED.swap(true, Ordering::Relaxed) {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::<Graphics>::with_user_event()
        .build()
        .expect("failed to create event loop");

    let proxy = event_loop.create_proxy();

    let app = App {
        proxy: Some(proxy),
        canvas_parent: Some("game-wrapper".to_string()),
        ..Default::default()
    };

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        let _ = event_loop.run_app(&mut app);
    }
}
