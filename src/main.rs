use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::iter::once_with;
use std::ops::{Add, AddAssign, Mul, Sub};
use std::panic;
use std::time::Instant;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use minifb::Key::{S, W};

const WIDTH: usize = 600;
const HEIGHT: usize = 400;

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

fn inc(window: &Window, inc: Key, dec: Key, increment: f32) -> f32 {
    if window.is_key_down(inc) {
        increment
    } else if window.is_key_down(dec) {
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

        if window.is_key_released(Key::NumPad4) {
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
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        frame_time = start.elapsed().as_micros()as f32/1000000.0;
        println!("Frame time: {}ms ({}fps)", frame_time*1000.0, 1.0/frame_time);
    }
}