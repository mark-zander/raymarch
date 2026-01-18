use wgpu::BindGroupEntry;
use wgpu::util::DeviceExt;
use num_traits::cast::ToPrimitive;

use enum_map::{enum_map, Enum, EnumMap};


// We need this for Rust to store our data correctly for the shaders
// #[repr(C)]
// This is so we can store this in a buffer
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// enum UniformType {
//     Int(i32),
//     Float(f32),
// }

//  Should this have a generic type parameter for value?
pub struct Uniform {
    name: String,           // Shader variable name
    value: i32,                 // Shader variable type
    bind_group: GroupIndex,
    binding: u32,
    buffer: wgpu::Buffer,
}

impl Uniform {
    fn new(
        name: &str,
        value: i32,
        bind_group: GroupIndex,
        binding: u32,
        device: &wgpu::Device,
    ) -> Self {
        // let name = name.to_string();
        // let buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some(&name),
        //         // contents: bytemuck::cast_slice(&[camera_uniform]),
        //         contents: bytemuck::cast_slice(&[i]),
        //         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     }
        // );
        // let binding_entry = wgpu::BindGroupEntry {
        //     binding: binding,
        //     resource: buffer.as_entire_binding(),
        // };
        Self {
            name: name.to_string(),
            value,
            bind_group,
            binding,
            buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&name),
                    // contents: bytemuck::cast_slice(&[camera_uniform]),
                    contents: bytemuck::cast_slice(&[value]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            ),
        }
    }
    fn make_buffer(
        &mut self,
        value: i32,
        device: &wgpu::Device,
    ) {
        self.value = value;
        self.buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&self.name),
                contents: bytemuck::cast_slice(&[value]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
    }
    fn make_layout(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    ////    should this be done in BindGroup? and prevent needing
    ///     to annotate lifetime?
    fn make_bind(
        &mut self,
        // device: &wgpu::Device,
    ) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.buffer.as_entire_binding(),
        }
    }

    fn make_wgsl(&self) -> String {
        let bind_goup = self.bind_group as u32;
        let binding = self.binding;
        let name = &self.name;
        format!("@group({bind_goup}) @binding({binding})
            var<uniform> {name}: i32;\n")
    }
}

pub struct BindGroup {
    pub bind_group: GroupIndex,
    pub uniforms: Vec<Uniform>,
    pub layouts: Vec<wgpu::BindGroupLayoutEntry>,
    // pub uniform_binds: Vec<wgpu::BindGroupEntry>,
    // pub layout:  wgpu::BindGroupLayoutDescriptor,
    // pub layout: wgpu::BindGroupLayout,
    // pub binding: wgpu::BindGroup,
}

impl BindGroup {
    fn new(bind_group: GroupIndex) -> Self {
        Self {
            bind_group,
            uniforms: Vec::new(),
            layouts: Vec::new(),
        }
    }
    fn new_uniform(
        &mut self,
        name: &str,
        ii: i32,
        // bind_group: u32,
        // binding: u32,
        device: &wgpu::Device,
    ) {
        let binding = self.uniforms.len().to_u32().expect("");
        let uniform = Uniform::new(
            name, ii, self.bind_group, binding, device);
        // self.uniforms.push(Uniform::new(
        //     name, ii, self.bind_group as u32, binding, device));
        self.layouts.push(uniform.make_layout());
        self.uniforms.push(uniform);
    }
    fn make_layout(
        &self,
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        // let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        // for uniform in &self.uniforms {
        //     layouts.push(uniform.make_layout());
        // }
        println!("uniforms length = {}", self.uniforms.len());
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                // entries: &self.layouts,
                entries: &self.layouts,
                label: Some(
                    &format!("{:?}_bind_group_layout", self.bind_group)),
            //.expect(&format!("not a bind group: {group_name}"));
            }
        )
    }
    fn make_binds(
        &mut self,
    ) -> Vec<wgpu::BindGroupEntry<'_>> {
        let mut binds: Vec<wgpu::BindGroupEntry> = Vec::new();
        // fill in here
        for uniform in &mut self.uniforms {
            binds.push(uniform.make_bind());
        }
        binds
    }
    fn make_group(
        &mut self,
        device: &wgpu::Device,
   ) -> wgpu::BindGroup {
        let bind_group = self.bind_group;
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &self.make_layout(device),
                entries: &self.make_binds(),
                // label: Some(&(self.name.clone() + "_bind_group")),
                label: Some(
                    &format!("{:?}_bind_group", bind_group)),
            }
        )
    }
    fn set_render_pass(
        &mut self,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass
    ) {
        let bind_group = self.bind_group as u32;
        render_pass.set_bind_group(
            bind_group, &self.make_group(device), &[]);
    }
    fn make_wgsl(&self) -> String {
        let mut str = String::new();
        for uniform in &self.uniforms {
            str.push_str(&uniform.make_wgsl());
        }
        str
    }
}   // BindGroup

#[derive(Debug, Enum, Clone, Copy)]
pub enum GroupIndex {
    Scalars=0,
    Textures,
}

fn group_names(index: GroupIndex) -> &'static str {
    match index  {
        GroupIndex::Scalars => { "Scalars" }
        GroupIndex::Textures => { "Textures" }
    }
}

//  Need all layouts struct?
// #[derive(Default)]
pub struct PipelineBindGroups {
    name: String,
    groups: EnumMap<GroupIndex, BindGroup>,
    layouts: Vec<wgpu::BindGroupLayout>,
    // list: GroupArray<BindGroup>,
    // list: [BindGroup; N_OBJECTS],
    // need a uniform name table (map)?
    // list: Vec<BindGroup>,
}

impl PipelineBindGroups {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            groups: enum_map!{
                GroupIndex::Scalars => 
                    BindGroup::new(GroupIndex::Scalars),
                GroupIndex::Textures =>
                    BindGroup::new(GroupIndex::Textures),
            },
            layouts: Vec::new(),
        }
    }
    pub fn new_uniform(
        &mut self,
        name: &str,
        group: GroupIndex,
        ii: i32,
        device: &wgpu::Device,
    ) {
        // let mut bind_group = self.list.last().expect("");
        // let mut bind_group = self.find_bind_group(group_name)
        //     .expect(&format!("not a bind group: {group_name}"));
        // bind_group.new_uniform(name, ii, device);
        println!("new uniform = {}", name);
        self.groups[group].new_uniform(name, ii, device);
    }
        // for uniform in &self.uniforms {
        //     layouts.push(uniform.make_layout());
        // }

    pub fn pipeline_layout(
        &mut self,
        device: &wgpu::Device
    ) -> wgpu::PipelineLayout {
        // Lives long enough for Rust but long enough for wgpu?
        // println!("{:#?}", &self.groups[GroupIndex::Scalars]);
        for (_k, g) in &self.groups {
            if !g.uniforms.is_empty() {
                self.layouts.push(g.make_layout(&device));
            }
        }

        let layout_ref: Vec<&wgpu::BindGroupLayout> =
            self.layouts.iter().collect();
        // let mut layout_ref: Vec<&wgpu::BindGroupLayout> = Vec::new();
        // for layout in &self.layouts {
        //     layout_ref.push(layout);
        // }
        let alayout = &layout_ref[..];
        println!("{:#?}", alayout);
        let aname = &self.name;
        device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                // label: Some(&(self.name.clone() + "_pipeline_layout")),
                label: Some(&format!("{aname}_pipeline_layout")),
                bind_group_layouts: &layout_ref[..],
                push_constant_ranges: &[],
            }
        )
    }
    pub fn set_render_pass(
        &mut self,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass,
    ) {
        for (_k, g) in &mut self.groups {
            g.set_render_pass(device, render_pass);
        }
    }
    pub fn make_wgsl(&self) -> String {
        let mut str = String::new();
        for (_k, g) in &self.groups {
            str.push_str(&g.make_wgsl());
        }
        str
    }

}