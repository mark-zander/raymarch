use std::sync::Arc;
mod uniform;
use crate:: uniform::*;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

// Should these be an enum in uniform.rs? No lookups needed.
// Errors at compile time.
const BINDINGS: &str = "bindings";
// const SCALARS: &str = "scalars";
// const TEXTURES: &str = "textures";

const SCREEN_X: &str = "screen_x";
const SCREEN_Y: &str = "screen_y";

// Event driven window handler for this application
#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    // last_size: winit::dpi::PhysicalSize<u32>,
}



impl ApplicationHandler for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let mut attributes = Window::default_attributes();
        attributes = attributes.with_title("Title");

        if let Ok(window) = event_loop.create_window(attributes) {
            let window_handle = Arc::new(window);
            self.window = Some(window_handle.clone());

            // Done in main? Better to have it here?
            // env_logger::init();

            let renderer = pollster::block_on(
                Renderer::new(window_handle.clone())
            );
            self.renderer = Some(renderer);

        }
    }

    fn window_event(
        &mut            self,
        event_loop:     &ActiveEventLoop,
        _id:            WindowId,
        event:          WindowEvent
    ) {
        let (Some(window), Some(renderer),) = (
            self.window.as_mut(),
            self.renderer.as_mut(),
        ) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.render();
                // Emits a new redraw requested event.
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                // gpu.resize(size);
                renderer.resize(size);
                // self.last_size = size;
            }
            _ => (),
        }
    }
}

// Creates a surface, device and queue for a window
// Allows public access to all of its fields for use by Renderer
pub struct Gpu {
    // window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    // size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
}

impl Gpu {
    async fn new(
        window: Arc<Window>,
        // bindings: &mut PipelineBindGroups
    ) -> Gpu {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor::default(),
                // None, // Trace path
            )
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Configure surface for the first time
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![surface_format.add_srgb_suffix()],
            width: size.width,
            height: size.height,
            present_mode: cap.present_modes[0],
            alpha_mode: cap.alpha_modes[0],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);
 

        Self {
            // window,
            device,
            queue,
            // size,
            surface,
            surface_config,
            surface_format,
        }
    }

    fn resize(
        &mut self,
        size: winit::dpi::PhysicalSize<u32>,
        // bindings: &mut PipelineBindGroups,
    ) {

        // reconfigure the surface
        // self.configure_surface(size);
        if size.width > 0 && size.height > 0 {
            // self.size = new_size;
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
            // Update screen size
            // bindings.set_uniform(
            //     "screen_x", size.width as i32, &self.device);
            // bindings.set_uniform(
            //     "screen_y", size.height as i32, &self.device);
        }
    }

    // pub fn create_depth_texture(
    //     &mut self, size: winit::dpi::PhysicalSize<u32>
    // ) -> wgpu::TextureView {
    // }
}

pub struct Renderer {
    gpu: Gpu,
    scene: Scene,
    size: winit::dpi::PhysicalSize<u32>,
    bindings: PipelineBindGroups,

    // depth_texture_view: wgpu::TextureView,
}

impl Renderer {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let gpu = Gpu::new(window).await;
        let mut bindings = PipelineBindGroups::new(BINDINGS);
        Self::init_bindings(&mut bindings, &size, &gpu.device);
        let scene = Scene::new(
            &gpu.device, gpu.surface_format, &mut bindings);
        Self {
            gpu,
            scene,
            size,
            bindings,
        }
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.gpu.resize(size);
    }

    fn init_bindings(
        bindings: &mut PipelineBindGroups,
        size: &winit::dpi::PhysicalSize<u32>,
        device: &wgpu::Device,
    ) {
         // Set the window size
        bindings.new_uniform(
            SCREEN_X, GroupIndex::Scalars, size.width as i32, device,
        );
        bindings.new_uniform(
            SCREEN_Y, GroupIndex::Scalars, size.height as i32, device,
        );

    }

    fn render(&mut self) {
        // Create texture view
        let surface_texture = self
            .gpu
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.gpu.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.scene.render(
            &mut renderpass, &self.gpu.device, &mut self.bindings);

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.gpu.queue.submit([encoder.finish()]);
        // self.gpu.window.pre_present_notify();
        surface_texture.present();
    }

}

struct Scene {
    pub pipeline: wgpu::RenderPipeline,
    // PipeLineBindGroups here?
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bindings: &mut PipelineBindGroups,
    ) -> Self {
        //  vertex buffer
        //  index buffer
        //  unifrom
        //  model
        let pipeline = Self::create_pipeline(
            device, surface_format, bindings);
        Self {
            pipeline,
        }
    }

    //  The values of PipeLineBindGroups are set here
    pub fn render<'rpass>(
        &'rpass self,
        renderpass: &mut wgpu::RenderPass<'rpass>,
        device: &wgpu::Device,
        pipeline_bind_groups: &mut PipelineBindGroups,
    ) {
        print!("{}", pipeline_bind_groups.make_wgsl());
        renderpass.set_pipeline(&self.pipeline);
        pipeline_bind_groups.set_render_pass(device, renderpass);
        // If you wanted to call any drawing commands, they would go here.
        renderpass.set_pipeline(&self.pipeline); // 2.
        renderpass.draw(0..6, 0..1); // 3.
    

        // renderpass.set_bind_group(0, &self.uniform.bind_group, &[]);

        // renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // renderpass.draw_indexed(0..(INDICES.len() as _), 0, 0..1);
    }

    // pub fn update(&mut self, queue: &wgpu::Queue, aspect_ratio: f32, delta_time: f32) {
    //     let projection =
    //         nalgebra_glm::perspective_lh_zo(aspect_ratio, 80_f32.to_radians(), 0.1, 1000.0);
    //     let view = nalgebra_glm::look_at_lh(
    //         &nalgebra_glm::vec3(0.0, 0.0, 3.0),
    //         &nalgebra_glm::vec3(0.0, 0.0, 0.0),
    //         &nalgebra_glm::Vec3::y(),
    //     );
    //     self.model = nalgebra_glm::rotate(
    //         &self.model,
    //         30_f32.to_radians() * delta_time,
    //         &nalgebra_glm::Vec3::y(),
    //     );
    //     self.uniform.update_buffer(
    //         queue,
    //         0,
    //         UniformBuffer {
    //             mvp: projection * view * self.model,
    //         },
    //     );
    // }

    //  The layouts of PipelineBindGroups are set here
    //  The layouts are around longer than the values!
    fn create_pipeline(
        device: &wgpu::Device,
        surface_config: wgpu::TextureFormat,
        pipeline_bind_groups: &mut PipelineBindGroups,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(
            wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
              pipeline_bind_groups.pipeline_layout(device);
        // let render_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("Render Pipeline Layout"),
        //         bind_group_layouts: &[],
        //         push_constant_ranges: &[],
        //     });

        // let render_pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: surface_config,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        })


    }

}
