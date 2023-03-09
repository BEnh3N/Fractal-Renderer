use std::time::Instant;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::{PhysicalSize};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Fullscreen};
use winit_input_helper::WinitInputHelper;

use rayon::prelude::*;

const FULLSCREEN: bool = true;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const MAX_ITERATION: usize = 500;

// const RATIO: f64 = 1.618033988749;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = PhysicalSize::new(WIDTH as f64, HEIGHT as f64);
        // let monitor = winit::monitor::MonitorHandle::;
        WindowBuilder::new()
            .with_title("Fractal")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    // Set window to fullscreen
    if FULLSCREEN {
        let monitor_handle = window.available_monitors().next();
        window.set_fullscreen(Some(Fullscreen::Borderless(monitor_handle)));
    }

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = Model::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame_mut());
            if let Err(err) = pixels.render() {
                dbg!("pixels.render() failed: {}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Space) {
                world.redraw = true;
            }

            if input.key_pressed(VirtualKeyCode::Up) {
                world.scale /= 10.;
                // world.scale
                world.redraw = true;
            }
            if input.key_pressed(VirtualKeyCode::Down) {
                world.scale *= 10.;
                // world.scale
                world.redraw = true;
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

/// Representation of the application state.
struct Model {
    redraw: bool,
    // theta: f64,
    constant: (f64, f64),
    scale: f64,
}

impl Model {
    fn new() -> Self {
        Self {
            redraw: true,
            constant: (-0.7269, 0.1889),
            scale: 1. / (HEIGHT as f64 / 2.),
        }
    }

    fn update(&mut self) {
        self.scale = self.scale * 0.90;
        self.redraw = true;
    }

    fn draw(&mut self, frame: &mut [u8]) {
        if self.redraw {
            let current = Instant::now();
            // Compute the scale of the coordinates
            frame.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                let x = (i % WIDTH as usize) as i16;
                let y = (i / WIDTH as usize) as i16;

                let samples = 8;
                let mut color = 0.;
                for _ in 0..samples {
                    // Compute pixel's coordinates
                    let px = ((x as f64 - WIDTH  as f64 / 2.) + rand::random::<f64>()) * self.scale;
                    let py = ((y as f64 - HEIGHT as f64 / 2.) + rand::random::<f64>()) * self.scale;
                    // Compute color
                    // let iterations = compute_iterations((0., 0.), (px, py), MAX_ITERATION);
                    let iterations = compute_iterations((px, py), self.constant, MAX_ITERATION);
                    color += iterations;
                }

                let g = (((color / samples as f64) / MAX_ITERATION as f64) * 255.) as u8;

                let rgba = [g, g, g, 0xff];
                pixel.copy_from_slice(&rgba);
            }); 
            dbg!(current.elapsed().as_secs_f32());
        }
        self.redraw = false;
    }
}

// Compute Zn² + C
fn compute_next(current: (f64, f64), constant: (f64, f64)) -> (f64, f64) {
    // Zn²
    let zr = current.0 * current.0 - current.1 * current.1;
    let zi = 2. * current.0 * current.1;

    // Add constant
    (zr + constant.0, zi + constant.1)
}

// Returns the absolute value
fn abs(z: (f64, f64)) -> f64 {
    z.0 * z.0 + z.1 * z.1
}

// Computes sequence elements until abs exceeds threshold or max iteration is reached
fn compute_iterations(z0: (f64, f64), constant: (f64, f64), max_iteration: usize) -> f64 {
    let mut zn = z0;
    let mut iteration = 0;
    while abs(zn) < 4. && iteration < max_iteration {
        zn = compute_next(zn, constant);
        iteration += 1;
    }

    let modi = abs(zn).sqrt();
    let smooth_iteration = iteration as f64 - (1_f64).max(modi.log2()).log2();
    smooth_iteration

    // iteration as f64
}
