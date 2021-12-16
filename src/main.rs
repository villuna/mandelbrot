use raylib::prelude::*;
use std::thread::{self, JoinHandle};

// Default values
const HUE: f32 = 252.;
const WIDTH: i32 = 1280;
const HEIGHT: i32 = 960;
const RANGE_X: (f64, f64) = (-2.0, 0.47);
const RANGE_Y: (f64, f64) = (-1.12, 1.12);
const MAX_ITERATIONS: i32 = 64;
const SPEED: f64 = 200.0;
const ZOOM: f64 = 0.9; // Sheogorath!
const ITER_SPEED: i32 = 64; // Add this many iterations per button press
const NO_THREADS: usize = 64;

const SIZE: usize = (WIDTH * HEIGHT) as usize;
const T_SIZE: usize = SIZE / NO_THREADS;

// Represents the position of the camera, and the dimensions of the viewport
// (in the complex plane, not in pixel space)
struct Camera {
    dim: [f64; 2],
    pos: [f64; 2],
    iterations: i32,
}

impl Camera {
    fn new(dim: [f64; 2], pos: [f64; 2], iterations: i32) -> Camera {
        Camera {
            dim,
            pos,
            iterations,
        }
    }

    fn update(&mut self, rl: &RaylibHandle) {
        let dt = rl.get_frame_time() as f64;
        // Update psoition
        // I know i misspelled that word but im keeping it cause its funni
        let hdir = (rl.is_key_down(KeyboardKey::KEY_D) as i32
            - rl.is_key_down(KeyboardKey::KEY_A) as i32) as f64;

        let vdir = (rl.is_key_down(KeyboardKey::KEY_S) as i32
            - rl.is_key_down(KeyboardKey::KEY_W) as i32) as f64;

        let dx = hdir * SPEED * dt / WIDTH as f64;
        let dy = vdir * SPEED * dt / HEIGHT as f64;

        self.pos[0] += dx * self.dim[0];
        self.pos[1] += dy * self.dim[1];

        // Change iteration level
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            self.iterations += ITER_SPEED;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            self.iterations -= ITER_SPEED;
        }

        self.iterations = self.iterations.max(0);

        // Change zoom level
        let scroll = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) as i32
            - rl.is_key_down(KeyboardKey::KEY_SPACE) as i32;
        if scroll > 0 {
            self.dim[0] *= ZOOM;
            self.dim[1] *= ZOOM;
        } else if scroll < 0 {
            self.dim[0] /= ZOOM;
            self.dim[1] /= ZOOM;
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            [RANGE_X.1 - RANGE_X.0, RANGE_Y.1 - RANGE_Y.0],
            [(RANGE_X.0 + RANGE_X.1) / 2., (RANGE_X.0 + RANGE_X.1) / 2.],
            MAX_ITERATIONS
        )
    }
}

// does ????
// stolen from wikipedia
fn mandelbrot(re: f64, im: f64, max_iterations: i32) -> u8 {
    let (mut x, mut y, mut x2, mut y2) = (0., 0., 0., 0.);
    let mut iterations = 0;

    while x*x + y*y <= 4.0 && iterations < max_iterations {
        y = 2.0 * x * y + im;
        x = x2 - y2 + re;
        x2 = x * x;
        y2 = y * y;
        iterations += 1;
    }

    ((iterations as f32 / max_iterations as f32) * 255.0) as u8
}

fn calc_range(index: i32, iterations: i32, dim: [f64; 2], pos: [f64; 2]) -> [u8; T_SIZE] {
    let row_size = HEIGHT / NO_THREADS as i32;
    let mut result = [0; T_SIZE];

    for j in 0..row_size {
        let jj = j + index * row_size;
        let y = ((jj - (HEIGHT / 2)) as f64) / HEIGHT as f64;
        let y = y * dim[1] + pos[1];

        for i in 0..WIDTH {
            let x = ((i - (WIDTH / 2)) as f64) / WIDTH as f64;
            let x = x * dim[0] + pos[0];

            result[(j * WIDTH + i) as usize] = mandelbrot(x, y, iterations);
        }
    }

    result
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("mandelbrot")
        .build();

    let mut c = Camera::default();

    let pallette: Vec<Color> = (0..256).map(|x| {
        Color::color_from_hsv(HUE, 1., x as f32 / 255.0)
    }).collect();

    let mut canvas = [0; SIZE];

    while !rl.window_should_close() {
        c.update(&rl);
        let mut threads: Vec<JoinHandle<[u8; T_SIZE]>> = Vec::new();

        for i in 0..NO_THREADS {
            let iterations = c.iterations;
            threads.push(thread::spawn(move || {
                calc_range(i as i32, iterations, c.dim, c.pos)
            }));
        }

        for (t, thread) in threads.into_iter().enumerate() {
            let res = thread.join().unwrap();

            for i in 0..T_SIZE {
                canvas[i + (t * T_SIZE)] = res[i];
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                let level = canvas[(j * WIDTH + i) as usize] as usize;
                let col = pallette[level];
                d.draw_pixel(i, j, col);
            }
        }


        d.draw_text(&format!("Iterations: {}", c.iterations), 10, 10, 30,
                    Color::WHITE);
        d.draw_fps(WIDTH - 100, 10);
    }
}
