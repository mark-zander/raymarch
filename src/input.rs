// Input handler for inputs from WindowEvent
// Many of the inputs interact with each other in this application
// such that they need to belong to a single state.
// use cgmath;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize}, event::*, keyboard::{KeyCode, ModifiersState},
};
use instant::{Duration, Instant};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2u ( [u32; 2] );

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
    // transform: Matrix4,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            duration: Duration::new(0, 0),
            frame_count: 0,
            screen: Vec2u([0, 0]),
            modifiers_state: ModifiersState::empty(),
            key_code: KeyCode::Abort,
            key_state: ElementState::Released,
            mouse_button: MouseButton::Other(0),
            mouse_state: ElementState::Released,
        }
    }
    pub fn new_frame(&mut self) {
        self.frame_count += 1;
        self.duration = self.start_time.elapsed();
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
    pub fn set_screen(&mut self, screen: winit::dpi::PhysicalSize<u32>) {
        self.screen = Vec2u([screen.width, screen.height]);
    }
    pub fn get_screen(&self) -> Vec2u  { self.screen }
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
        println!("state = {key_state:?}, button = {key_code:?}");
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

    }
    pub fn handle_pan(
        &mut self,
        phase: TouchPhase,
        delta: PhysicalPosition<f32>
    ) {

    }
    pub fn handle_rotation(&mut self, phase: TouchPhase, delta: f32) {

    }
}
