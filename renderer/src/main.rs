use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::iter::once_with;
use std::ops::{Add, AddAssign, Mul, Sub};
use std::panic;
//use std::time::Instant;
/*use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use minifb::Key::{S, W};*/

use error_iter::ErrorIter as _;
use log::error;
use pixels::{PixelsBuilder, SurfaceTexture};
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
//use winit_input_helper::WinitInputHelper;


const WIDTH: u32 = 600;
const HEIGHT: u32 = 400;

struct Renderer {
    vertices: Vec<Vector3>,
    camera_pos: Vector3,
    projection_plane_distance: (f32),
    camera_angle: f32,
}

impl Renderer {
    pub fn project_vertices(&self, height: i32, width: i32) -> Vec<(f32, f32)> {
        let mut projected_vertices = Vec::<(f32, f32)>::new();
        for vert in self.vertices.iter() {
            let vertex = (*vert).clone();

            let cam_space_pos = vertex - self.camera_pos.clone();

            let cam_forward = self.get_cam_forward();
            let cam_right = self.get_cam_right();

            let rotated_pos = cam_right * cam_space_pos.x + cam_forward * cam_space_pos.z + Vector3::UP * cam_space_pos.y;
            if(rotated_pos.z < 0.0){ continue;}
            let projected_pos_x = rotated_pos.x / rotated_pos.z * &self.projection_plane_distance;
            let projected_pos_y = rotated_pos.y / rotated_pos.z * &self.projection_plane_distance;
            let projected_pos = (projected_pos_x, projected_pos_y);

            let transformed_pos = Self::transform_to_window_space(projected_pos, width, height);

            projected_vertices.push(transformed_pos);
        }
        projected_vertices
    }

    fn transform_to_window_space(pos: (f32, f32), width: i32, height: i32) -> (f32, f32) {
        (pos.0 * 500.0 + (width as f32 / 2.0), pos.1 * -500.0 + (height as f32 / 2.0))
    }

    pub fn get_cam_forward(&self) -> Vector3 {
        Vector3::new(self.camera_angle.to_radians().sin(), 0.0, self.camera_angle.to_radians().cos())
    }

    pub fn get_cam_right(&self) -> Vector3 {
        Vector3::new(self.camera_angle.to_radians().cos(), 0.0, -self.camera_angle.to_radians().sin())
    }

    pub fn get_cam_up(&self) -> Vector3 {
        Vector3::UP // TODO
    }


    fn draw(&self, frame: &mut [u8]){
        let mut index = 0;

        const fac: f32 = 500f32;
        let vertices: Vec<(f32, f32)> = self.project_vertices(HEIGHT as i32, WIDTH as i32);
        let vert_ref = &vertices;
        //println!("Projection took {}ms", start.elapsed().as_micros()as f32/1000.0);

        //let bufNow = Instant::now();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let mut is_at_vertex = false;
            for i in 0..vertices.len(){
                let vertex = &vertices[i];

                let dist_x = x.abs_diff((vertex.0 as u32).try_into().unwrap());
                let dist_y = y.abs_diff((vertex.1 as u32).try_into().unwrap());
                is_at_vertex |= dist_x < 5 && dist_y < 5;
                //is_at_vertex |= x == vertex.0 as u32 && y == vertex.1 as u32;
                if is_at_vertex {
                    break;
                }
            }


            let (r, g, b) = if is_at_vertex{
                (255u8, 255u8, 255u8)
            }else{
                (((x as i32) as f32 / (WIDTH as f32 / 255.0)) as u8, (y as f32 / (HEIGHT as f32 / 255.0)) as u8, 0u8)

            };
            let rgba = [r, g, b, 0xff];

            //*i = (r as u32) << 16 | (g as u32) << 8 | (b as u32);

            pixel.copy_from_slice(&rgba);
        }
    }
}

struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Clone for Vector3 {
    fn clone(&self) -> Self {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Vector3 {
    const UP: Vector3 = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 {
            x,
            y,
            z,
        }
    }
}
impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Display for Vector3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*format!("{} {} {}", &self.x, &self.y, &self.z))
    }
}

struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex {
    fn position_as_tuple(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}

struct State {
    vertices: Vec<Vertex>,
}

impl State {
    fn new(vertices: Vec<Vertex>) -> State {
        State {
            vertices
        }
    }
}
struct FragData {
    x: u32,
    y: u32,
}

fn frag(data: FragData, state: &State) -> (u8, u8, u8) {
    panic!()
}

fn inc(inputHelper: &WinitInputHelper, inc: KeyCode, dec: KeyCode, increment: f32) -> f32 {
    if inputHelper.key_held(inc) {
        increment
    } else if inputHelper.key_held(dec) {
        -increment
    } else {
        0f32
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}


fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        pollster::block_on(run());
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

#[cfg(target_arch = "wasm32")]
/// Retrieve current width and height dimensions of browser client window
fn get_window_size() -> LogicalSize<f64> {
    let client_window = web_sys::window().unwrap();
    LogicalSize::new(
        client_window.inner_width().unwrap().as_f64().unwrap(),
        client_window.inner_height().unwrap().as_f64().unwrap(),
    )
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + Web")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas().unwrap()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new({
            let window = Rc::clone(&window);
            move |_e: web_sys::Event| {
                let _ = window.request_inner_size(get_window_size());
            }
        }) as Box<dyn FnMut(_)>);
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();

        // Trigger initial resize event
        let _ = window.request_inner_size(get_window_size());
    }

    let mut input = WinitInputHelper::new();
    let mut pixels = {
        #[cfg(not(target_arch = "wasm32"))]
        let window_size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        let window_size = get_window_size().to_physical::<u32>(window.scale_factor());

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        let builder = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture);

        #[cfg(target_arch = "wasm32")]
        let builder = {
            // Web targets do not support the default texture format
            let texture_format = pixels::wgpu::TextureFormat::Rgba8Unorm;
            builder
                .texture_format(texture_format)
                .surface_texture_format(texture_format)
        };

        builder.build_async().await.expect("Pixels error")
    };
    let mut renderer = Renderer {
        vertices: vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0),
        ],
        camera_pos: Vector3::new(0.0, 0.0, -5.0),
        projection_plane_distance: 1.0,
        camera_angle: 0.0,
    };
    let res = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Draw the current frame
                renderer.draw(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    elwt.exit();
                    return;
                }

                // Update internal state and request a redraw
                //world.update();
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Resize the window
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    elwt.exit();
                    return;
                }
            }

            _ => (),
        }

        // Handle input events
        if input.update(&event) {



            let frame_time = 1.0/140.0;
            const incr: f32 = 10.0;
            // I need to invert the x-axis of cam_forward and the z-axis of cam-right for camera movement to work correctly, but I have no clue why
            let _fwd = renderer.get_cam_forward();
            let fwd = Vector3::new(-_fwd.x, _fwd.y, _fwd.z);
            let _rgt = renderer.get_cam_right();
            let rgt =  Vector3::new(_rgt.x, _rgt.y, -_rgt.z);

            renderer.camera_pos +=
                rgt * inc(&input, KeyCode::KeyD, KeyCode::KeyA, incr) * frame_time
                    + fwd * inc(&input, KeyCode::KeyW, KeyCode::KeyS, incr) * frame_time
                    + renderer.get_cam_up() * inc(&input, KeyCode::KeyI, KeyCode::KeyK, incr) * frame_time;
            renderer.camera_angle += inc(&input, KeyCode::KeyQ, KeyCode::KeyE, 100.0) * frame_time;
            renderer.projection_plane_distance += inc(&input, KeyCode::KeyN, KeyCode::KeyM, 0.01);



            #[cfg(not(target_arch = "wasm32"))]
            if (input.key_pressed(KeyCode::Escape) || input.close_requested()) {
                elwt.exit();
            }
        }
    });
    res.unwrap();
}

/*fn mainZ() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Graph",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: true,
            ..WindowOptions::default()
        },
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.set_target_fps(144);

    let mut renderer = Renderer {
        vertices: vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0),
        ],
        camera_pos: Vector3::new(0.0, 0.0, -5.0),
        projection_plane_distance: 1.0,
        camera_angle: 0.0,
    };
    //let mut state = State::default();

    let mut frame_time: f32 = 1.0/60.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = Instant::now();

        let mut index = 0;

        const fac: f32 = 500f32;
        let vertices: Vec<(f32, f32)> = renderer.project_vertices(HEIGHT as i32, WIDTH as i32);
        let vert_ref = &vertices;
        //println!("Projection took {}ms", start.elapsed().as_micros()as f32/1000.0);

        let bufNow = Instant::now();
        for i in buffer.iter_mut() {
            let x = index % WIDTH as u32;
            let y = index / WIDTH as u32;

            /*let frag_data = FragData {
                x,
                y,
            };
            */

            let mut is_at_vertex = false;
            for i in 0..vertices.len(){
                let vertex = &vertices[i];

                let dist_x = x.abs_diff(vertex.0 as u32);
                let dist_y = y.abs_diff(vertex.1 as u32);
                is_at_vertex |= dist_x < 5 && dist_y < 5;
                //is_at_vertex |= x == vertex.0 as u32 && y == vertex.1 as u32;
                if is_at_vertex {
                    break;
                }
            }


            let (r, g, b) = if is_at_vertex{
                (255u8, 255u8, 255u8)
            }else{
                (((x as i32) as f32 / (WIDTH as f32 / 255.0)) as u8, (y as f32 / (HEIGHT as f32 / 255.0)) as u8, 0u8)

            };

            *i = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
            index += 1
        }
        //println!("Buffer edition took {}ms", bufNow.elapsed().as_micros()as f32/1000.0);
        /*
        state.a += inc(&window, W, S, 5f32);
        state.b += inc(&window, Key::A, Key::D, 0.0005f32);
        */

        let f = 1.0/144.0;
        const incr: f32 = 10.0;
        // I need to invert the x-axis of cam_forward and the z-axis of cam-right for camera movement to work correctly, but I have no clue why
        let _fwd = renderer.get_cam_forward();
        let fwd = Vector3::new(-_fwd.x, _fwd.y, _fwd.z);
        let _rgt = renderer.get_cam_right();
        let rgt =  Vector3::new(_rgt.x, _rgt.y, -_rgt.z);

        renderer.camera_pos +=
            rgt * inc(&window, Key::D, Key::A, incr) * frame_time
                + fwd * inc(&window, Key::W, Key::S, incr) * frame_time
                + renderer.get_cam_up() * inc(&window, Key::I, Key::K, incr) * frame_time;
        renderer.camera_angle += inc(&window, Key::Q, Key::E, 100.0) * frame_time;
        renderer.projection_plane_distance += inc(&window, Key::N, Key::M, 0.01);

/*        if window.is_key_released(Key::NumPad4) {
            renderer.camera_angle += 90.0;
        }
        if window.is_key_released(Key::NumPad6) {
            renderer.camera_angle += -90.0;
        }
        if window.is_key_released(Key::NumPad8) {
            renderer.camera_angle += 0.0;
        }
        if window.is_key_released(Key::NumPad2) {
            renderer.camera_angle += 180.0;
        }*/

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        frame_time = start.elapsed().as_micros()as f32/1000000.0;
        println!("Frame time: {}ms ({}fps)", frame_time*1000.0, 1.0/frame_time);
    }
}*/