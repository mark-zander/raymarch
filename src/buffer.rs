use bytemuck::AnyBitPattern;
use nalgebra::Vector3;
use wgpu::{self, BindGroupLayout};
use wgpu::BindGroupEntry;
use wgpu::util::DeviceExt;
use flume::bounded;

// Takes data supplied by inputs and passes it on to the GPU.
fn uniform_layout(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry{
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
}

fn storage_layout_r(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry{
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_usage_r() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
}

fn storage_layout_rw(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry{
        binding,
        // In Device::create_bind_group_layout,
        // label = 'input_uniforms_layout'
        // Binding 6 entry is invalid
        // Features Features {
        // features_wgpu: FeaturesWGPU(VERTEX_WRITABLE_STORAGE),
        // features_webgpu: FeaturesWebGPU(0x0) }
        // are required but not enabled on the device
        // visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_usage_rw() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE
    | wgpu::BufferUsages::COPY_DST
    | wgpu::BufferUsages::COPY_SRC
}

pub struct BufferSetup {
    pub layout: fn(u32) -> wgpu::BindGroupLayoutEntry,
    pub usage: fn() -> wgpu::BufferUsages,
}

pub static UNIFORM_SET: BufferSetup = BufferSetup {
    layout: uniform_layout,
    usage: uniform_usage,
};

// Read only for storage buffers
pub static STORAGE_SET_R: BufferSetup = BufferSetup {
    layout: storage_layout_r,
    usage: storage_usage_r,
};

// Read/Write for storage buffers
pub static STORAGE_SET_RW: BufferSetup = BufferSetup {
    layout: storage_layout_rw,
    usage: storage_usage_rw,
};

pub struct Buffer {
    pub name: String,
    pub n_bind: u32,
    pub bufr: wgpu::Buffer,
    pub setup: &'static BufferSetup,
}

impl Buffer {
    pub fn new(
        name: &str,
        n_bind: u32,
        bufr: wgpu::Buffer,
        setup: &'static BufferSetup,
    ) -> Self {
        Self { name: name.to_string(), n_bind, bufr, setup }
    }
    fn layout(&self) -> wgpu::BindGroupLayoutEntry {
        (self.setup.layout)(self.n_bind)
    }
    fn bind(&self) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: self.n_bind,
            resource: self.bufr.as_entire_binding(),
        }
    }
}

pub struct BufferPod<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub bufr: Buffer,
    pub data: T,
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> BufferPod<T> {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        n_bind: u32,
        data: T,
        setup: &'static BufferSetup,
    ) -> Self {
        let bufr = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: bytemuck::cast_slice(&[data]),
                usage: (setup.usage)(),
            }
        );
        Self {
            bufr: Buffer::new(name, n_bind, bufr, setup),
            data,
        }
    }
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        // let uniforms = &mut self.uniforms;
        queue.write_buffer(
            &self.bufr.bufr,
            0,
            bytemuck::cast_slice(&[self.data])
        );
    }
}

pub struct BufferVec<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub bufr: Buffer,
    pub data: Vec<T>,
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> BufferVec<T> {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        n_bind: u32,
        data: Vec<T>,
        setup: &'static BufferSetup,
    ) -> Self {
        let bufr = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: bytemuck::cast_slice(&data),
                usage: (setup.usage)(),
            }
        );
        Self {
            bufr: Buffer::new(name, n_bind, bufr, setup),
            data,
        }
    }
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        // let uniforms = &mut self.uniforms;
        queue.write_buffer(
            &self.bufr.bufr,
            0,
            bytemuck::cast_slice(&self.data)
        );
    }
}

// Data transfer GPU to CPU of a single data type element
pub struct BufferPodW<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable + std::fmt::Debug
    + std::cmp::Eq,
{
    pub bufr: Buffer,
    pub tbufr: wgpu::Buffer,
    pub data: T,
    // pub prev: T,
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable + std::fmt::Debug
    + std::cmp::Eq>
BufferPodW<T> {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        n_bind: u32,
        data: T,
        setup: &'static BufferSetup,
    ) -> Self {
        // A single data type has the same size as that data
        // type in an array of one.
        // println!(
        //     "size of [T; 1] = {:?}, size of T = {:?}",
        //     size_of::<[T; 1]>(),
        //     size_of::<T>(),
        // );
        let bufr = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some(name),
                size: std::mem::size_of::<T>() as u64,
                usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }
        );
        let tbufr = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("temp"),
            size: bufr.size(),
            usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            bufr: Buffer::new(name, n_bind, bufr, setup),
            tbufr,
            data,
            // prev: data,
        }
    }
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        // let uniforms = &mut self.uniforms;
        queue.write_buffer(
            &self.bufr.bufr,
            0,
            bytemuck::cast_slice(&[self.data])
        );
    }
    pub fn initiate_from_gpu(
        &self, encoder: &mut wgpu::CommandEncoder
    ) {
        encoder.copy_buffer_to_buffer(
            &self.bufr.bufr, 0, &self.tbufr, 0, self.bufr.bufr.size()
        );
    }
    pub async fn get_from_gpu(
        &mut self,
        device: &wgpu::Device,
        // t0: T,
    ) -> anyhow::Result<()> {
        {
            // The mapping process is async, so we'll need to create a channel to get
            // the success flag for our mapping
            let (tx, rx) = bounded(1);

            // We send the success or failure of our mapping via a callback
            self.tbufr.map_async(wgpu::MapMode::Read, .., move |result| {
                tx.send(result).unwrap()
            });

            // The callback we submitted to map async will only get called after the
            // device is polled or the queue submitted
            device.poll(wgpu::PollType::wait_indefinitely())?;

            // We check if the mapping was successful here
            rx.recv_async().await??;

            // We then get the bytes that were stored in the buffer
            let output = self.tbufr.get_mapped_range(..)?;
            let adata = bytemuck::cast_slice::<_, T>(&output);
            // if adata[0] != t0 && adata[0] != self.prev {
            if adata[0] != self.data {
                // this prints too often. print only on value change
                println!("from gpu = {:?}", adata[0]);
                self.data = adata[0];
            }
            // Now we have the data on the CPU we can do what ever we want to with it
            // assert_eq!(&[self.data], bytemuck::cast_slice(&output));
            // println!("from gpu = {:?}", bytemuck::cast_slice(&output));
        }
        // We need to unmap the buffer to be able to use it again
        self.tbufr.unmap();

        Ok(())
    }
}



pub struct Group {
    // Want to be able to print the var used by shader but,
    // This doesn't work, ownership issues.
    // pub buffers: Vec<&'a Buffer>,
    pub layout: wgpu::BindGroupLayout,
    pub bind: wgpu::BindGroup,
}

impl Group {
    pub fn new(
        device: &wgpu::Device, label: &str, buffers: &[&Buffer]
    ) -> Self {
        let mut some_layouts = Vec::with_capacity(buffers.len());
        let mut some_binds = Vec::with_capacity(buffers.len());
        for buf in buffers {
            some_layouts.push(buf.layout());
            some_binds.push(buf.bind());
        }
        let layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &some_layouts,
                label: Some(&format!("{label}_layout")),
            }
        );
        let bind = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &some_binds,
                label: Some(&format!("{label}_bind")),
            }
        );
        Self {
            layout,
            bind,
        }
    }
    fn print_vars(&self) -> String {
        " ".to_string()
    }
}