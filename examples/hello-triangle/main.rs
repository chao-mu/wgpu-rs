use std::mem;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

use zerocopy::{AsBytes, FromBytes};

#[path = "../framework.rs"]
mod framework;

#[repr(C)] 
#[derive(Clone, Copy, AsBytes, FromBytes, Debug)]
struct Uniforms {
    color: [f32; 3],
    size: [f32; 2]
}


async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let surface = wgpu::Surface::create(&window);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
        },
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    })
    .await;
    
    let vs_bytes =
        framework::load_glsl(include_str!("shader.vert"), framework::ShaderStage::Vertex);
    let fs_bytes =
        framework::load_glsl(include_str!("shader.frag"), framework::ShaderStage::Fragment);
    let vs_module = device.create_shader_module(&vs_bytes);
    let fs_module = device.create_shader_module(&fs_bytes);

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: None,
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::RedrawRequested(_) => {
                let size = window.inner_size();
                let uniforms = Uniforms {
                    color: [0.0, 1.0, 0.5],
                    size: [size.width as f32, size.height as f32]
                };
                let uniform_size = mem::size_of::<Uniforms>() as wgpu::BufferAddress;
                let uniform_buf = device.create_buffer_with_data(
                    uniforms.as_bytes(),
                    wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                );
                //println!("{:?}", uniforms);

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &uniform_buf,
                                range: 0 .. uniform_size,
                            },
                        },
                    ],
                    label: None,
                });

                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color::GREEN,
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.draw(0 .. 6, 0 .. 1);
                }

                queue.submit(&[encoder.finish()]);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    env_logger::init();
    futures::executor::block_on(run(event_loop, window));
}
