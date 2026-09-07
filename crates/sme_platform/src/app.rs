//! Thin native/web app runner. Game hosts own simulation, input routing and rendering;
//! this module owns window creation, one-time async initialization and event-loop setup.
//! No GPU, audio, game schema or scripting dependency belongs here.
use crate::window::{create_window, PlatformConfig};
use std::{future::Future, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

pub trait GameHost: Sized + 'static {
    fn create(window: Arc<Window>) -> impl Future<Output = Self>;
    fn window(&self) -> &Window;
    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
}

struct Runner<G: 'static> {
    config: PlatformConfig,
    state: Option<G>,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    proxy: EventLoopProxy<G>,
    init_started: bool,
    pending_focus: Option<bool>,
}

impl<G: GameHost> Runner<G> {
    fn install(&mut self, event_loop: &ActiveEventLoop, mut state: G) {
        // The surface may have changed size while async initialization was in flight.
        let size = state.window().inner_size();
        state.event(event_loop, WindowEvent::Resized(size));
        if let Some(focused) = self.pending_focus.take() {
            state.event(event_loop, WindowEvent::Focused(focused));
        }
        state.window().request_redraw();
        self.state = Some(state);
    }
}

impl<G: GameHost> ApplicationHandler<G> for Runner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.init_started {
            return;
        }
        self.init_started = true;
        let window = create_window(event_loop, &self.config);
        #[cfg(not(target_arch = "wasm32"))]
        self.install(event_loop, pollster::block_on(G::create(window)));
        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = proxy.send_event(G::create(window).await);
            });
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, state: G) {
        self.install(event_loop, state);
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window().request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(state) = &mut self.state {
            if id == state.window().id() {
                state.event(event_loop, event);
            }
        } else {
            match event {
                WindowEvent::Focused(focused) => self.pending_focus = Some(focused),
                WindowEvent::CloseRequested => event_loop.exit(),
                _ => {} // Gameplay input during loading is deliberately not replayed.
            }
        }
    }
}

/// Run on the native main thread; on web this schedules the app and returns.
/// Initialization errors currently follow the host's policy (existing hosts panic
/// on unavailable GPU). This is not yet a suspend/resume surface-recreation layer.
pub fn run<G: GameHost>(config: PlatformConfig) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::<G>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let app = Runner {
        config,
        state: None,
        proxy: event_loop.create_proxy(),
        init_started: false,
        pending_focus: None,
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop.run_app(&mut app)
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
        Ok(())
    }
}
