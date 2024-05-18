use wgpu::{
    BindGroupLayoutEntry, BindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

pub struct BindingBuilder {
    binding: u32,
    stage: Option<ShaderStages>,
    binding_type: Option<BindingType>,
}

impl BindingBuilder {
    pub fn new(binding: u32) -> Self {
        Self {
            binding,
            stage: None,
            binding_type: None,
        }
    }

    pub fn at_stages(mut self, stage: ShaderStages) -> Self {
        self.stage.replace(stage);
        self
    }
    pub fn texture_entry(
        mut self,
        dim: TextureViewDimension,
        sample_type: TextureSampleType,
    ) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: self.binding,
            visibility: self.shader_stages(),
            ty: BindingType::Texture {
                sample_type,
                view_dimension: dim,
                multisampled: false,
            },
            count: None,
        }
    }
    pub fn uniform_entry(mut self) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: self.binding,
            visibility: self.shader_stages(),
            ty: BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
    pub fn sampler_entry(mut self) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: self.binding,
            visibility: self.shader_stages(),
            ty: BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        }
    }

    fn shader_stages(&mut self) -> ShaderStages {
        self.stage.expect("need shader-stages")
    }
    pub fn with_entry_type(mut self, binding_type: BindingType) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: self.binding,
            visibility: self.shader_stages(),
            ty: binding_type,
            count: None,
        }
    }
}
