use wgpu::{
  util::{BufferInitDescriptor, DeviceExt},
  *,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 2],
  uv: [f32; 2],
}

impl Vertex {
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Circle {
  pub position: [f32; 2],
  pub radius: f32,
  pub color: [u8; 4],
}

impl Circle {
  fn desc() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 2,
          format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
          shader_location: 3,
          format: wgpu::VertexFormat::Float32,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 4,
          format: wgpu::VertexFormat::Uint32,
        },
      ],
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
  width: f32,
  height: f32,
}

const SQUARE: &[Vertex] = &[
  Vertex {
    position: [-1.0, 1.0],
    uv: [0.0, 0.0],
  },
  Vertex {
    position: [1.0, 1.0],
    uv: [1.0, 0.0],
  },
  Vertex {
    position: [1.0, -1.0],
    uv: [1.0, 1.0],
  },
  Vertex {
    position: [1.0, -1.0],
    uv: [1.0, 1.0],
  },
  Vertex {
    position: [-1.0, -1.0],
    uv: [0.0, 1.0],
  },
  Vertex {
    position: [-1.0, 1.0],
    uv: [0.0, 0.0],
  },
];

pub struct QuickDraw {
  device: Device,
  queue: Queue,
  width: u32,
  height: u32,
  uniform_buffer: Buffer,
  uniform_bind_group: BindGroup,
  texture_desc: TextureDescriptor<'static>,
  texture: Texture,
  texture_view: TextureView,
  instance_count: u64,
  instance_buffer: Buffer,
  output_buffer: Buffer,
  vertex_buffer: Buffer,
  pipeline: RenderPipeline,
}

impl QuickDraw {
  pub async fn new(width: u32, height: u32, max_circles: u64) -> Self {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    let adapter = match instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        ..Default::default()
      })
      .await
    {
      Some(a) => a,
      None => {
        eprintln!("Could not find WGPU adapter");
        std::process::exit(1);
      },
    };
    let (device, queue) =
      match adapter.request_device(&Default::default(), None).await {
        Ok(r) => r,
        Err(e) => {
          eprintln!("Could not find WGPU device:");
          eprintln!("{}", e);
          std::process::exit(1);
        },
      };

    let uniforms = GpuUniforms {
      width: width as f32,
      height: height as f32,
    };
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Uniform Buffer"),
      contents: bytemuck::cast_slice(&[uniforms]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_layout =
      device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
        label: Some("Uniform Bind Group Layout"),
      });

    let uniform_bind_group =
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_layout,
        entries: &[BindGroupEntry {
          binding: 0,
          resource: uniform_buffer.as_entire_binding(),
        }],
      });

    let texture_desc = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT,
      label: None,
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());

    let instance_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Instance Buffer"),
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
      mapped_at_creation: false,
      size: max_circles * std::mem::size_of::<Circle>() as u64,
    });

    let u32_size = std::mem::size_of::<u32>() as u32;
    let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      size: output_buffer_size,
      usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
      label: None,
      mapped_at_creation: true,
    });
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      usage: BufferUsages::VERTEX,
      contents: bytemuck::cast_slice(SQUARE),
    });
    // Shaders
    let shader = std::borrow::Cow::Borrowed(include_str!("shader.wgsl"));
    let vert = device.create_shader_module(ShaderModuleDescriptor {
      label: Some("Vertex Shader"),
      source: ShaderSource::Wgsl(shader.clone()),
    });
    let frag = device.create_shader_module(ShaderModuleDescriptor {
      label: Some("Vertex Shader"),
      source: ShaderSource::Wgsl(shader),
    });
    // Pipeline
    let render_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&uniform_layout],
        push_constant_ranges: &[],
      });
    let pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        multiview: None,
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &vert,
          entry_point: "vs_main",
          buffers: &[Vertex::desc(), Circle::desc()],
        },
        fragment: Some(wgpu::FragmentState {
          module: &frag,
          entry_point: "fs_main",
          targets: &[Some(wgpu::ColorTargetState {
            format: texture_desc.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
          })],
        }),
        primitive: wgpu::PrimitiveState {
          topology: wgpu::PrimitiveTopology::TriangleList,
          strip_index_format: None,
          front_face: wgpu::FrontFace::Ccw,
          cull_mode: None,
          unclipped_depth: false,
          conservative: false,
          // Setting this to anything other than Fill requires
          // Features::NON_FILL_POLYGON_MODE
          polygon_mode: wgpu::PolygonMode::Fill,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
          count: 1,
          mask: !0,
          alpha_to_coverage_enabled: false,
        },
      });

    Self {
      device,
      queue,
      width,
      height,
      uniform_buffer,
      uniform_bind_group,
      texture_desc,
      texture,
      texture_view,
      instance_count: 0,
      instance_buffer,
      output_buffer,
      vertex_buffer,
      pipeline,
    }
  }

  pub async fn resize(&mut self, width: u32, height: u32, max_circles: usize) {
    let texture_desc = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT,
      label: None,
    };
    let texture = self.device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());

    let u32_size = std::mem::size_of::<u32>() as u32;
    let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
    let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
      size: output_buffer_size,
      usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
      label: None,
      mapped_at_creation: true,
    });
    self.allocate(max_circles).await;
    self.width = width;
    self.height = height;
    self.texture_desc = texture_desc;
    self.texture = texture;
    self.texture_view = texture_view;
    self.output_buffer = output_buffer;
  }

  async fn draw_call(&mut self) {
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let render_pass_desc = wgpu::RenderPassDescriptor {
      occlusion_query_set: None,
      timestamp_writes: None,
      label: Some("Render Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &self.texture_view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
          }),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
    };
    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    render_pass.draw(0..6, 0..self.instance_count as u32);
    drop(render_pass);

    let u32_size = std::mem::size_of::<u32>() as u32;

    encoder.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        aspect: wgpu::TextureAspect::All,
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
      },
      wgpu::ImageCopyBuffer {
        buffer: &self.output_buffer,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(u32_size * self.width),
          rows_per_image: Some(self.height),
        },
      },
      self.texture_desc.size,
    );
    self.output_buffer.unmap();
    self.queue.submit(Some(encoder.finish()));
  }

  async fn read_output_buffer(&self) -> BufferView {
    let buffer_slice = self.output_buffer.slice(..);

    // NOTE: We have to create the mapping THEN device.poll()
    // before await the future. Otherwise the application
    // will freeze.
    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
      tx.send(result).unwrap();
    });
    // buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
    self.device.poll(wgpu::Maintain::Wait);
    rx.receive().await.unwrap().unwrap();
    buffer_slice.get_mapped_range()
  }

  // Allocates space for *size* circles, also clears the
  // circles buffer
  pub async fn allocate(&mut self, size: usize) {
    let size = (size * std::mem::size_of::<Circle>()) as u64;
    if size >= self.instance_buffer.size() {
      self.instance_buffer = self.device.create_buffer(&BufferDescriptor {
        label: Some("Instance Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size,
      });
    }
  }

  async fn write_circles(&mut self, circles: &[Circle]) {
    self.allocate(circles.len()).await;
    self.queue.write_buffer(
      &self.instance_buffer,
      0,
      &bytemuck::cast_slice(&circles),
    );
    // self.queue.submit([]);
    self.instance_count = circles.len() as u64;
  }

  async fn bytes(&self) -> Vec<u8> {
    let mut buffer = vec![0; self.output_buffer.size() as usize];
    {
      let data = self.read_output_buffer().await;

      for (i, d) in data.iter().enumerate() {
        buffer[i] = *d;
      }
    }
    buffer
  }

  pub async fn draw_circles(&mut self, circles: &[Circle]) -> Vec<u8> {
    self.write_circles(circles).await;
    self.draw_call().await;
    self.bytes().await
  }
}
