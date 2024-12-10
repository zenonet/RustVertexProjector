use std::panic;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use minifb::Key::{S, W};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;


struct FragData {
    x: u32,
    y: u32,
}

struct State {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}
impl State {
    fn default() -> State {
        State {
            a: 100f32,
            b: 0.01f32,
            c: 150f32,
            d: 0f32,
        }
    }
}

fn frag(data: FragData, state: &State) -> (u8, u8, u8) {
    let sin = (data.x as f32 * state.b).sin() * state.a + state.c + (data.x as f32 / 30f32 * state.b + 0.5f32).sin() * 200f32;

    if (sin - data.y as f32).abs() < 2f32 {
        (255, 255, 255)
    } else {
        (0, 0, 0)
    }
}

fn inc(window: &Window, inc:Key, dec:Key, increment:f32) -> f32{
    if window.is_key_down(inc){
        increment
    }else if window.is_key_down(dec){
        -increment
    }else{
        0f32
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Graph",
        WIDTH,
        HEIGHT,
        WindowOptions{
            ..WindowOptions::default()
        },
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    let mut state = State::default();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut index = 0;
        for i in buffer.iter_mut() {
            let x = index % WIDTH as u32;
            let y = index / WIDTH as u32;

            let frag_data = FragData {
                x,
                y,
            };

            let (r, g, b) = frag(frag_data, &state);

            *i = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
            index += 1
        }

        state.a += inc(&window, W, S, 5f32);
        state.b += inc(&window, Key::A, Key::D, 0.0005f32);


        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}


