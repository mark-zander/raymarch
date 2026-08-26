use bytemuck::AnyBitPattern;
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
use std::{f32::consts::TAU};
extern crate nalgebra as na;

use crate::buffer::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2u ( [u32; 2] );
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2f ( [f32; 2] );
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec4f ( [f32; 4] );
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

const ID4X4F: Mat4x4f =
    Mat4x4f([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

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
    mouse_hit: u32,
    mouse_pos: Vec2u,
    cursor: Vec2u,
    transformer: Vec<Matrix4f>,
    scale: Vec<f32>,
    node_id: usize,
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
            mouse_hit: 0,
            mouse_pos: Vec2u([0, 0]),
            cursor: Vec2u([0, 0]),
            transformer: vec!(Matrix4f::identity(); 4),
            scale: vec!(1.0; 4),
            node_id: 0,
        }
    }
    pub fn new_frame(
        &mut self,
        queue: &wgpu::Queue,
        uniforms: &mut Uniforms
    ) {
        self.frame_count += 1;
        self.duration = self.start_time.elapsed();
        self.node_id = uniforms.node_id.data as usize;
        uniforms.timer.data = self.duration.as_secs_f32();
        // uniforms.scale.data.0[self.node_id] = self.scale[self.node_id];
        uniforms.scale.data[self.node_id] = self.scale[self.node_id];
        uniforms.mouse_hit.data = self.mouse_hit;
        uniforms.mouse_pos.data = self.mouse_pos;
        uniforms.cursor.data = self.cursor;
        uniforms.transformer.data[self.node_id].0 =
            self.transformer[self.node_id].data.0;
        // uniforms.node_id.data = self.node_id as u32;
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
    pub fn resize(
        &mut self,
        queue: &wgpu::Queue,
        uniforms: &mut Uniforms,
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
                        self.transformer[self.node_id] *=
                        Matrix4f::from_axis_angle(
                            &Vector3::x_axis(), delta
                        );
                    }
                    KeyCode::ArrowDown => {
                        self.transformer[self.node_id] *=
                        Matrix4f::from_axis_angle(
                            &Vector3::x_axis(), -delta
                        );
                    }
                    KeyCode::ArrowLeft => {
                        self.transformer[self.node_id] *=
                        Matrix4f::from_axis_angle(
                            &Vector3::y_axis(), delta
                        );
                    }
                    KeyCode::ArrowRight => {
                        self.transformer[self.node_id] *=
                        Matrix4f::from_axis_angle(
                            &Vector3::y_axis(), -delta
                        );
                    }
                    KeyCode::Digit0 => { self.node_id = 0; }
                    KeyCode::Digit1 => { self.node_id = 1; }
                    KeyCode::Digit2 => { self.node_id = 2; }
                    _ => (),
                }
            }
            ElementState::Released => {}
        }
    }
    pub fn handle_cursor(
        &mut self,
        position: PhysicalPosition<f64>
    ) {
        self.cursor = Vec2u([position.x as u32, position.y as u32]);
    }
    pub fn handle_mouse(
        &mut self,
        state: ElementState,
        button: MouseButton
    ) {
        match button {
            MouseButton::Left => {
                self.mouse_hit = match state {
                    ElementState::Released => 0,
                    ElementState::Pressed => {
                        self.mouse_pos = self.cursor;
                        1
                    },
                }
            }
            _ => (),
        }
        println!(
            "{}, cursor = {:?}, screen = {:?}",
            self.mouse_hit, self.cursor, self.screen
        );
    }
    pub fn handle_pinch(&mut self, phase: TouchPhase, delta: f64) {
        if delta.is_finite() {
            let a_scale = 1.0 + delta as f32;
            // println!("{phase:?}, {a_scale}");
            if phase == TouchPhase::Moved {
                self.transformer[self.node_id] *= scale(a_scale);
                self.scale[self.node_id] *= a_scale;
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
            self.transformer[self.node_id] *=
            translate(Vector3f::new(x, y, 0.0));
        }
    }
    pub fn handle_rotation(&mut self, phase: TouchPhase, delta: f32) {
        if phase == TouchPhase::Moved {
            self.transformer[self.node_id] *=
            Matrix4f::from_axis_angle(
                &Vector3::z_axis(), -delta
            );
        }
    }
}

// const UNIFORMS: &str = "uniforms";
const SCREEN_XY: &str = "screen_xy";
const TRANSFORMER: &str = "transformer";
const SCALE: &str = "scale";
const TIMER: &str = "timer";
const MOUSE_HIT: &str = "mouse_hit";
const CURSOR: &str = "cursor";
const NODE_ID: &str = "node_id";
const MOUSE_POS: &str = "mouse_pos";
const INPUT_UNIFORMS: &str = "input_uniforms";

pub struct Uniforms {
    pub screen_xy: BufferPod<Vec2u>,
    pub timer: BufferPod<f32>,
    pub transformer: BufferVec<Mat4x4f>,
    // pub scale: BufferPod<Vec4f>,
    pub scale: BufferVec<f32>,
    pub mouse_hit: BufferPod<u32>,
    pub cursor: BufferPod<Vec2u>,
    pub node_id: BufferPodW<u32>,
    pub mouse_pos: BufferPod<Vec2u>,
    pub group: Group,
}

impl Uniforms {
    pub fn new(
        device: &wgpu::Device,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let data = Vec2u([size.width, size.height]);
        let screen_xy = BufferPod::<Vec2u>::new(
            device, SCREEN_XY, 0, data, &UNIFORM_SET
        );
        let timer = BufferPod::<f32>::new(
            device, TIMER, 1, 0.0, &UNIFORM_SET
        );
        let data = vec!(ID4X4F; 4);
        let transformer = BufferVec::<Mat4x4f>::new(
            device, TRANSFORMER, 2, data, &STORAGE_SET_R
        );
        let scale = BufferVec::<f32>::new(
            device, SCALE, 3, vec!(1.0; 4), &STORAGE_SET_R
        );
        let mouse_hit = BufferPod::<u32>::new(
            device, MOUSE_HIT, 4, 0, &UNIFORM_SET
        );
        let cursor = BufferPod::<Vec2u>::new(
            device, CURSOR, 5, Vec2u([0, 0]), &UNIFORM_SET
        );
        let node_id = BufferPodW::<u32>::new(
            device, NODE_ID, 6, 0, &STORAGE_SET_RW
        );
        let mouse_pos = BufferPod::<Vec2u>::new(
            device, MOUSE_POS, 7, Vec2u([0, 0]), &UNIFORM_SET
        );
        let group = Group::new(
            device,
            INPUT_UNIFORMS,
            &[
                &screen_xy.bufr,
                &timer.bufr,
                &transformer.bufr,
                &scale.bufr,
                &mouse_hit.bufr,
                &cursor.bufr,
                &node_id.bufr,
                &mouse_pos.bufr,
            ]
        );

        Self {
            screen_xy,
            timer,
            transformer,
            scale,
            mouse_hit,
            cursor,
            node_id,
            mouse_pos,
            group,
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
        self.timer.update(queue);
        self.transformer.update(queue);
        self.scale.update(queue);
        self.mouse_pos.update(queue);
        // self.node_id.update(queue);
    }
}
