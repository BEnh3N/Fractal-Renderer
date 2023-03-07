use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1920/2;
const HEIGHT: u32 = 1080/2;
const MAX_ITERATION: usize = 500;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Fractal")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

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

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

/// Representation of the application state.
struct Model {
    redraw: bool,
    constant: (f64, f64),
}

impl Model {
    fn new() -> Self {
        Self {
            redraw: true,
            constant: (-0.8, 0.156),
        }
    }

    fn update(&mut self) {
        // self.constant.0 += 0.01;
        // self.constant.1 -= 0.01;
    }

    fn draw(&mut self, frame: &mut [u8]) {
        if self.redraw {
            dbg!("Redraw Request");
            // Compute the scale of the coordinates
            let scale = 1. / (HEIGHT as f64 / 2.);

            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let x = (i % WIDTH as usize) as i16;
                let y = (i / WIDTH as usize) as i16;

                let samples = 16;
                let mut color = 0.;
                for _ in 0..samples {
                    // Compute pixel's coordinates
                    let px = ((x as f64 - WIDTH  as f64 / 2.) + rand::random::<f64>()) * scale;
                    let py = ((y as f64 - HEIGHT as f64 / 2.) + rand::random::<f64>()) * scale;
                    // Compute color
                    let iterations = compute_iterations((px, py), self.constant, MAX_ITERATION);
                    color += iterations;
                }

                // let iterations = compute_iterations((0., 0.), (px, py), SAMPLES);
                let g = (((color / samples as f64) / MAX_ITERATION as f64) * 255.) as u8;
                // let g = (iterations * 255 as f64) as u8;
                let rgba = [g, g, g, 0xff];

                pixel.copy_from_slice(&rgba);
            }
            self.redraw = false;
        }
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
