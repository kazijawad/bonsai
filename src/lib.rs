mod renderer;
mod vertex;

use renderer::Renderer;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("Pat");
    window.set_inner_size(LogicalSize::new(1000.0, 1000.0));
    window.set_resizable(false);

    let mut renderer = Renderer::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(physical_size) => {
                renderer.resize(physical_size);
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(*new_inner_size);
                window.request_redraw();
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => match renderer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => {
                renderer.resize(renderer.size);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                *control_flow = ControlFlow::Exit;
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        },
        _ => {}
    });
}
