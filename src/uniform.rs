use nalgebra::Vector3;
use wgpu;
use wgpu::BindGroupEntry;
use wgpu::util::DeviceExt;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    keyboard::{KeyCode, ModifiersState},
};
use instant::{Duration, Instant};
use std::f32::consts::TAU;
extern crate nalgebra as na;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2u ( [u32; 2] );
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mat4x4f ( [[f32; 4]; 4] );

type Matrix4f = na::Matrix4<f32>;
type Vector3f = na::Vector3<f32>;

// &Vector3::z_axis(), TAU / 8.0
fn rotate(axis: &na::Unit<Vector3f>, angle: f32) -> Matrix4f {
    let theta = -angle;
    Matrix4f::from_axis_angle(axis, angle)
}

fn scale(s: f32) -> Matrix4f {
    let s1 = 1.0 / s;
    Matrix4f::new(
        s1,  0.0, 0.0, 0.0,
        0.0, s1,  0.0, 0.0,
        0.0, 0.0, s1,  0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

fn translate(v: Vector3f) -> Matrix4f {
    let v1 = -v;
    Matrix4f::new(
        1.0, 0.0, 0.0, v.x,
        0.0, 1.0, 0.0, v.y,
        0.0, 0.0, 1.0, v.z,
        0.0, 0.0, 0.0, 1.0,
    )
}

// Two structs that are tightly coupled. InputHandler takes input
// from WindowEvents and processes it for struct Uniform that then
// passes the data to the GPU.

// Input handler for inputs from WindowEvent
// Many of the inputs interact with each other in this application
// such that they need to belong to a single state.
// use cgmath;
pub struct InputHandler {
    start_time: Instant,
    duration: Duration,
    frame_count: u64,
    screen: Vec2u,
    modifiers_state: ModifiersState,
    key_code: KeyCode,
    key_state: ElementState,
    mouse_button: MouseButton,
    mouse_state: ElementState,
    // cursor: Point2,
    transformer: Matrix4f,
    scale: f32,
}

impl InputHandler {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        let data = Vec2u([size.width, size.height]);
        Self {
            start_time: Instant::now(),
            duration: Duration::new(0, 0),
            frame_count: 0,
            screen: data,
            modifiers_state: ModifiersState::empty(),
            key_code: KeyCode::Abort,
            key_state: ElementState::Released,
            mouse_button: MouseButton::Other(0),
            mouse_state: ElementState::Released,
            transformer: Matrix4f::identity(),
            scale: 1.0,
        }
    }
    pub fn new_frame(
        &mut self,
        queue: &wgpu::Queue,
        uniforms: &mut Uniform
    ) {
        self.frame_count += 1;
        self.duration = self.start_time.elapsed();
        uniforms.timer.data = self.duration.as_secs_f32();

        // self.transformer = Matrix4f::from_axis_angle(
        //     &Vector3::z_axis(), TAU / 8.0
        // );
        // self.transformer = translate(Vector3f::new(2.0, 0.0, 0.0));
        // let a_scale = 0.5;
        // self.transformer = scale(a_scale);
        // if self.frame_count == 1 {
        //     println!("scale = {}", self.scale);
        //     println!("{}", self.transformer);
        //     match self.transformer.try_inverse() {
        //         Some(inverse) => println!("{}", inverse),
        //         None => {}
        //     }
        // }

        uniforms.scale.data = self.scale;
        uniforms.transformer.data.0 = self.transformer.data.0;
        uniforms.update(queue);
    }
    // Currently prints out the time stats, should it return them?
    pub fn stats(&self) {
        let duration = self.duration;
        let frame_count = self.frame_count;
        let frames_sec = frame_count as f64 / duration.as_secs_f64();
        println!(
            "duration = {duration:?}, frame_count = {frame_count}, \
            frames per second = {frames_sec}"
        );
    }
    // pub fn set_screen(&mut self, screen: winit::dpi::PhysicalSize<u32>) {
    //     self.screen = Vec2u([screen.width, screen.height]);
    // }
    // pub fn get_screen(&self) -> Vec2u  { self.screen }
    pub fn resize(
        &mut self,
        queue: &wgpu::Queue,
        uniforms: &mut Uniform,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        self.screen = Vec2u([size.width, size.height]);
        uniforms.screen_xy.data = self.screen;
        uniforms.resize(queue);
    }
    pub fn handle_modifiers(
        &mut self,
        modifiers_state: ModifiersState,
    ) {
        self.modifiers_state = modifiers_state;
    }
    pub fn handle_key(
        &mut self, 
        key_state: ElementState,
        key_code: KeyCode
    ) {
        // println!("state = {key_state:?}, button = {key_code:?}");
        match key_state {
            ElementState::Pressed => {
                let delta = TAU / 60.0;
                match key_code {
                    KeyCode::ArrowUp => {
                        self.transformer *= Matrix4f::from_axis_angle(
                            &Vector3::x_axis(), delta
                        );
                    }
                    KeyCode::ArrowDown => {
                        self.transformer *= Matrix4f::from_axis_angle(
                            &Vector3::x_axis(), -delta
                        );
                    }
                    KeyCode::ArrowLeft => {
                        self.transformer *= Matrix4f::from_axis_angle(
                            &Vector3::y_axis(), delta
                        );
                    }
                    KeyCode::ArrowRight => {
                        self.transformer *= Matrix4f::from_axis_angle(
                            &Vector3::y_axis(), -delta
                        );
                    }
                    _ => (),
                }
            }
            ElementState::Released => {}
        }
    }
    pub fn handle_cursor(
        &mut self,
        max_position: PhysicalSize<u32>,
        position: PhysicalPosition<f64>
    ) {

    }
    pub fn handle_mouse(
        &mut self,
        state: ElementState,
        button: MouseButton
    ) {
        println!("state = {state:?}, button = {button:?}");
    }
    pub fn handle_pinch(&mut self, phase: TouchPhase, delta: f64) {
        if delta.is_finite() {
            let a_scale = 1.0 + delta as f32;
            // println!("{phase:?}, {a_scale}");
            if phase == TouchPhase::Moved {
                self.transformer *= scale(a_scale);
                self.scale *= a_scale;
            }
        }
    }
    pub fn handle_pan(
        &mut self,
        phase: TouchPhase,
        delta: MouseScrollDelta,
    ) {
        if phase == TouchPhase::Moved {
            // println!("{phase:?}, {delta:?}");
            let (x, y) = match delta {
                MouseScrollDelta::LineDelta(y, x) => (x, y),
                MouseScrollDelta::PixelDelta(p) => (
                    2.0 * -p.x as f32 / self.screen.0[0] as f32,
                    2.0 * p.y as f32 / self.screen.0[1] as f32
                )
            };
            // println!("x = {x}, y = {y}");
            self.transformer *= translate(Vector3f::new(x, y, 0.0));
        }
    }
    pub fn handle_rotation(&mut self, phase: TouchPhase, delta: f32) {
        if phase == TouchPhase::Moved {
            self.transformer *= Matrix4f::from_axis_angle(
                &Vector3::z_axis(), -delta
            );
        }
    }
}

const UNIFORMS: &str = "uniforms";
const SCREEN_XY: &str = "screen_xy";
const TRANSFORMER: &str = "transformer";
const SCALE: &str = "scale";
const TIMER: &str = "timer";

// Takes data supplied by inputs and passes it on to the GPU.
const SIMPLE_LAYOUT: wgpu::BindGroupLayoutEntry =
wgpu::BindGroupLayoutEntry{
    binding: 0,
    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
};

fn uniform_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
}

pub struct Auniform<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub name: String,
    pub n_bind: u32,
    pub data: T,
    pub buff: wgpu::Buffer,
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> Auniform<T> {
    fn new(
        device: &wgpu::Device,
        name: &str,
        n_bind: u32,
        data: T
    ) -> Self {
        Self {
            name: name.to_string(),
            n_bind,
            data,
            buff: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(name),
                    contents: bytemuck::cast_slice(&[data]),
                    usage: uniform_usage(),
                }
            )
        }
    }
    fn layout(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.n_bind,
            ..SIMPLE_LAYOUT
        }
    }
    fn bind(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.n_bind,
            resource: self.buff.as_entire_binding(),
        }
    }
    fn update(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        // let uniforms = &mut self.uniforms;
        queue.write_buffer(
            &self.buff,
            0,
            bytemuck::cast_slice(&[self.data])
        );
    }
}

pub struct Uniform {
    pub screen_xy: Auniform<Vec2u>,
    pub transformer: Auniform<Mat4x4f>,
    pub timer: Auniform<f32>,
    pub scale: Auniform<f32>,
    pub uniform_group_layout: wgpu::BindGroupLayout,
    pub uniform_group: wgpu::BindGroup,
}

impl Uniform {
    pub fn new(
        device: &wgpu::Device,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let data = Vec2u([size.width, size.height]);
        let screen_xy = Auniform::<Vec2u>::new(
            device, SCREEN_XY, 0, data
        );
        let data = Mat4x4f([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let transformer = Auniform::<Mat4x4f>::new(
            device, TRANSFORMER, 1, data
        );
        let timer = Auniform::<f32>::new(
            device, TIMER, 2, 0.0
        );
        let scale = Auniform::<f32>::new(
            device, SCALE, 3, 1.0
        );
        let uniform_group_layout =
            device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    screen_xy.layout(),
                    transformer.layout(),
                    timer.layout(),
                    scale.layout(),
                ],
                label: Some(&format!("{UNIFORMS}_group")),
            });

        let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_group_layout,
            entries: &[
                screen_xy.bind(),
                transformer.bind(),
                timer.bind(),
                scale.bind(),
            ],
            label: Some(UNIFORMS),
        });

        Self {
            screen_xy,
            transformer,
            timer,
            scale,
            uniform_group_layout,
            uniform_group,
        }
    }
    pub fn resize(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        self.screen_xy.update(queue);
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        self.transformer.update(queue);
        self.timer.update(queue);
        self.scale.update(queue);
    }
}