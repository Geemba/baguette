use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard};

use crate::*;
use input::winit::{event_loop::ActiveEventLoop, window::{Window, WindowAttributes}};

pub struct Renderer<'a>(&'a mut RendererData);

impl<'a> From<&'a mut RendererData> for Renderer<'a> 
{
    fn from(data: &'a mut RendererData) -> Self
    {
        Self(data)
    }
}

impl Renderer<'_>
{
    pub(crate) fn ctx(&self) -> &ContextHandle
    {
        &self.0.ctx
    }

    pub fn ui(&self) -> ui::Ui
    {
        (&self.0.ui).into()
    }

    /// todo: make option to create camera
    pub fn get_camera(&mut self) -> Camera
    {
        self.0.camera.clone()
    }
    
    /// loads a sprite from a [SpriteBuilder] to be rendered,
    pub fn add_sprite(&mut self, sprite: SpriteBuilder) -> Sprite
    {
        let ctx = self.ctx().clone();

        let renderpasses = self.0.passes
            .get_or_insert_with(Default::default);

        renderpasses.add_sprite(ctx, sprite)

    }

    pub fn add_tilemap(&mut self, tilemap: impl Into<TilemapBuilder>)
    {
        let ctx = self.0.ctx.read();

        let renderpasses = self.0.passes
            .get_or_insert_with(Default::default);

        renderpasses.add_tilemap(&ctx, tilemap.into())
    }

    /// returns the screen size in the format you decide,
    /// ex:
    /// ```
    /// app.screen_size::<f32>()
    /// ```
    pub fn screen_size<T>(&self) -> (T,T)
        where T: input::winit::dpi::Pixel
    {
        use input::winit::dpi::Pixel;

        let (width, heigth) = self.0.ctx.read().screen.size();
        (width.cast(), heigth.cast())
    }
    
    pub fn set_background_color(&mut self, r: f64, g: f64, b: f64)
    {
        self.0.set_clear_color(r, g, b)
    }
}

/// this is handled by the engine
pub struct RendererData
{
    ctx: ContextHandle,
    clear_color: wgpu::Color,

    /// the window that the renderer draws on
    pub window: Option<Arc<Window>>,
    pub ui: ui::UiData,

    /// attributes used when creating a window.
    w_attributes: WindowAttributes,
    camera: Camera,   
    adapter: wgpu::Adapter,
    passes: Option<RenderPassCommands>,
    output: FrameOutput,

}

// integration specific
impl RendererData
{
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64)
    {
        self.clear_color = wgpu::Color
        {
            r,
            g,
            b,
            a: 1.0,
        }
    }

    fn camera(&mut self) -> std::cell::RefMut<CameraData>
    {
        self.camera.data.borrow_mut()
    }

    pub fn resize(&mut self, (width, height): (u32,u32))
    {

        let mut ctx_write = self.ctx.0.write();
        ctx_write.screen.config.width = width;
        ctx_write.screen.config.height = height;

        drop(ctx_write);

        let ctx_read = self.ctx.0.read();      

        let (physical_width, physical_height) = 
        (
            width as f32, height as f32
        );

        self.ui.update_screen_size(width, height);
        self.output.update_texture(&ctx_read.device, width, height);

        if let Some(passes) = &mut self.passes
        {
            passes.resize(&ctx_read)
        }

        drop(ctx_read);

        // resize camera to match new screen size
        self.camera().resize(physical_width / physical_height);

        self.update_surface()
    }

    /// returns the render of this [`Renderer`].
    ///
    /// # Errors
    ///
    /// this function will return an error if the surface is unable to be retrieved.
    pub fn render
    (
        &mut self,
        window_target: &input::winit::event_loop::ActiveEventLoop,
    ) -> Result<(), wgpu::SurfaceError>
    {
        let ctx = self.ctx.read();

            self.camera.data.borrow_mut().update(&ctx);

        let camera = &self.camera.data.borrow();

        let output = ctx.screen.surface
            .as_ref()
            .expect("unexpected render call without an active surface")
            .get_current_texture()?;

        let frame_output_view = &output.texture.create_view(&Default::default());

        let mut encoder = ctx.create_command_encoder("render encoder");
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
            {
                label: Some("renderer pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment
                {
                    view: &self.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations
                    {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            });

            if let Some(passes) = &mut self.passes 
            {
                passes.prepare(&ctx);
                passes.draw(&ctx, &mut pass, camera);
            }

            self.ui.render
            (
                &mut pass,
                self.window.as_ref().unwrap(),
                window_target,
                &ctx
            )
        }

        self.output.copy_to(&mut encoder, frame_output_view);
        
        ctx.queue.submit([encoder.finish()]);
        output.present();
        
        Ok(())
    }

    /// simply renders a solid color
    ///
    /// # Errors
    ///
    /// this function will return an error if the surface is not able to be retrieved.
    pub fn render_plain_color(&mut self, r:f64,g:f64,b:f64) -> Result<(), wgpu::SurfaceError>
    {
        let ctx_read = self.ctx.read();

        //CameraData::update_all();
        self.camera.data.borrow_mut().update(&ctx_read);
    
        let output = ctx_read.screen.surface.as_ref().unwrap().get_current_texture()?;
        let frame_output = &output.texture.create_view(&Default::default());

        let mut encoder = ctx_read.create_command_encoder("render encoder");
        
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor
        {
            label: Some("renderer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment
            {
                view: frame_output,
                resolve_target: None,
                ops: wgpu::Operations
                {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1. }),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None
        });
        
        ctx_read.queue.submit([encoder.finish()]);
        output.present();
        
        Ok(())
    }

    /// can be used to release resources after rendering
    pub fn post_render(&mut self)
    {
        if self.passes.is_some() 
        {    
            self.ui.free_textures()
        }        
    }

    pub fn suspend(&mut self)
    {
        self.ctx.0.write().screen.destroy();
    }

    /// required to be called for any change to [wgpu::Device] to be effective.
    /// 
    /// will update the surface with new config values
    ///
    fn update_surface(&mut self)
    {
        let ctx_read = self.ctx.read();
        ctx_read.screen.surface
            .as_ref()
            .unwrap()
            .configure
            (
                &ctx_read.device, &ctx_read.screen.config
            );
    }

    /// list all limits that were requested of this device.
    /// if any of these limits are exceeded, functions may panic.
    pub fn limits(&self) -> wgpu::Limits
    {
        self.ctx.0.read().device.limits()
    }

    pub fn begin_egui_frame(&mut self)
    {
        self.ui.begin_egui_frame(self.window.as_ref().unwrap())
    }

    /// Creates a new [`Renderer`].
    ///
    /// # Panics
    ///
    /// panics if an appropriate adapter or device is not avaiable.
    #[must_use]
    pub fn new(w_attributes: WindowAttributes, color: Option<(f64,f64,f64)>) -> Self
    {   
        use wgpu::*;

        let backends = match cfg!(target_os = "windows")
        {
            true => Backends::DX12,
            false => Backends::PRIMARY
        };

        let instance = Instance::new(InstanceDescriptor
        {
            backends,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions
        {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None
        })).expect("bruh failed to find an appropriate adapter");

        #[cfg(debug_assertions)]
        {
            use owo_colors::*;

            let info = adapter.get_info();
            
            log::info!
            (
                "{} {{ backend: {}, using {:?}: {} }}",
                "Adapter".blue(),
                info.backend,
                info.device_type,
                info.name,
                
            );
        }

        // features that must be enabled for the app to work
        let required_features = Features::TEXTURE_BINDING_ARRAY
        | Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
        | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

        let (device, queue) = pollster::block_on
        (
            adapter.request_device
            (
                &DeviceDescriptor
                {
                    label: Some("renderer device"),
                    required_features,
                    required_limits: adapter.limits(),
                }, 
                None
            )
        ).expect("bruh failed to retrieve a device");

            // width and height of the rendered area in pixels
            let (width,height) = (1,1);

            // scalefactor of the screen we are rendering inside of
            let scale = 1.;

        let output = FrameOutput::new(&device,width,height);
        
        let ctx_data = ContextHandleInner::new(instance, device, queue);

        let ui = ui::UiData::new(&ctx_data, width,height,scale);

        let camera = Camera
        {
            data: std::cell::RefCell::new
            (
                CameraData::new(&ctx_data)
            ).into()
        };

        let ctx = ContextHandle(RwLock::new(ctx_data).into());

        let clear_color = color.map(|(r,g,b)| wgpu::Color
        {
            r,
            g,
            b,
            a: 1.0
        }
        ).unwrap_or(wgpu::Color
        {
            r: 0.13,
            g: 0.31,
            b: 0.85,
            a: 1.0
        });

        Self
        {
            adapter,
            passes: None,

            window: None,
            w_attributes,

            ui,
            camera,
            output,
            ctx,
            clear_color,
        }
    }

    /// this is where the window actually starts getting rendered.
    ///
    /// # Panics
    ///
    /// panics if the surface is not capable of being created.
    pub fn resume(&mut self, event_loop: &ActiveEventLoop)
    {
        use wgpu::*;

        let window = Arc::new
        (
            event_loop.create_window(self.w_attributes.clone())
            .expect("failed to create window")
        );

        let surface = self.ctx.read().instance.create_surface(window.clone())
            .expect("failed to create surface on window");
        
        self.window = Some(window);

        let surface_caps = surface.get_capabilities(&self.adapter);

        //preferably srgb format
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&surface_caps.formats[0]);

        let present_mode = PresentMode::Fifo;

        let config = SurfaceConfiguration
        {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width: 1, height: 1, present_mode,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        ////

        self.ctx.0.write().screen = Screen::new(surface, config);
        self.update_surface()
    }

    /// returns the backend of the adapter
    pub fn backend(&self) -> wgpu::Backend
    {
        self.adapter.get_info().backend
    }
}

/// handles how to present the final texture
struct FrameOutput
{
    view: wgpu::TextureView,
    bindgroup: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}
impl FrameOutput
{
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self 
    {
        use wgpu::*;

        let module = &device.create_shader_module
        (
            ShaderModuleDescriptor
            {
                label: None,
                source: ShaderSource::Wgsl
                (
                    include_str!("shaders/tex_to_tex_copy.wgsl").into()
                )
            }
        );

        let bindgroup_layout = &device.create_bind_group_layout
        (
            &BindGroupLayoutDescriptor
            {
                label: None,
                entries: &[BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture 
                    {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },BindGroupLayoutEntry
                {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None
                }]
            }
        );

        let view = device.create_texture
        (
            &TextureDescriptor
            {
                label: Some("output texture"),
                size: Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }
        ).create_view(&Default::default());

        let sampler = device.create_sampler
        (
            &SamplerDescriptor 
            {
                label: Some("output sampler"),
                ..Default::default()
            }
        );

        let bindgroup = device.create_bind_group
        (
            &BindGroupDescriptor
            {
                label: Some("frame output bindgroup"),
                layout: bindgroup_layout,
                entries: &[BindGroupEntry
                {
                    binding: 0,
                    resource: BindingResource::TextureView(&view)
                },BindGroupEntry
                {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler)
                }]
            }
        );

        Self
        {
            bindgroup,
            pipeline: device.create_render_pipeline
            (
                &RenderPipelineDescriptor
                {
                    label: Some("output render pipeline"),
                    layout: Some(&device.create_pipeline_layout
                    (
                        &PipelineLayoutDescriptor
                        {
                            label: Some("output pipeline descriptor"),
                            bind_group_layouts: &[bindgroup_layout],
                            push_constant_ranges: &[]
                        }
                    )),
                    vertex: VertexState
                    {
                        module, entry_point: "vertex", buffers:
                        &[
                            VertexBufferLayout
                            {
                                array_stride: std::mem::size_of::<[f32;5]>() as _,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                            }
                        ],
                        compilation_options: Default::default(),
                    },
                    primitive: PrimitiveState
                    {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Cw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: PolygonMode::Fill,
                        conservative: false
                    },
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some
                    (
                        FragmentState 
                        {
                            module,
                            entry_point: "fragment",
                            targets: &[Some(ColorTargetState
                            {
                                format: TextureFormat::Bgra8UnormSrgb,
                                blend: Some(BlendState::REPLACE),
                                write_mask: ColorWrites::ALL
                            })],
                            compilation_options: Default::default(),
                        }
                    ),
                    multiview: None
                }
            ),
            vertex_buffer: wgpu::util::DeviceExt::create_buffer_init
            (
                device,
                &wgpu::util::BufferInitDescriptor
                {
                    label: None,
                    contents: bytemuck::cast_slice::<[f32;5], u8>
                    (
                        &[
                           [-1., 1., 0., 0., 0.],
                           [-1., -1., 0., 0., 1.],
                           [1., -1., 0., 1., 1.],

                           [1., -1., 0., 1., 1.],
                           [-1., 1., 0., 0., 0.],
                           [1., 1., 0., 1., 0.],
                        ]
                    ),
                    usage: BufferUsages::VERTEX
                }
            ),
            view,
            sampler
        }
    }

    /// update the texture to match the width and height arguments
    fn update_texture(&mut self, device: &wgpu::Device, width: u32,height: u32)
    {
        self.view = device.create_texture
        (
            &wgpu::TextureDescriptor
            {
                label: Some("output texture"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }
        ).create_view(&Default::default());

        let bindgroup_layout = &device.create_bind_group_layout
        (
            &wgpu::BindGroupLayoutDescriptor
            {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture 
                    {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },wgpu::BindGroupLayoutEntry
                {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None
                }]
            }
        );

        self.bindgroup = device.create_bind_group
        (
            &wgpu::BindGroupDescriptor
            {
                label: Some("frame output bindgroup"),
                layout: bindgroup_layout,
                entries: &[wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view)
                },wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler)
                }]
            }
        );  
    }

    /// copy output to this texture
    pub fn copy_to(&self, encoder: &mut wgpu::CommandEncoder, dest: &wgpu::TextureView)
    {
        let mut pass = encoder.begin_render_pass
        (
            &wgpu::RenderPassDescriptor
            {
                label: Some("copy texture renderpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment
                {
                    view: dest,
                    resolve_target: None,
                    ops: wgpu::Operations
                    {
                        load: Default::default(),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            }
        );

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bindgroup, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        pass.draw(0..6, 0..1)
    }
}

#[derive(Clone)]
pub struct ContextHandle(Arc<RwLock<ContextHandleInner>>);

impl ContextHandle
{
    pub fn read(&self) -> RwLockReadGuard<ContextHandleInner>
    {
        match cfg!(debug_assertions)
        {
            // panic on timeout on debug
            true => self.0.try_read_for(std::time::Duration::from_secs(1))
                .expect("renderer context timeout reached"),

            false => self.0.read()
        }
    }
}

pub struct Screen
{
    pub surface: Option<wgpu::Surface<'static>>,
    pub config: wgpu::SurfaceConfiguration
}

impl Screen
{
    pub fn new(surface: wgpu::Surface<'static>, config : wgpu::SurfaceConfiguration) -> Self
    {
        Self
        {
            surface: Some(surface),
            config,
        }
    }

    pub fn destroy(&mut self)
    {
        self.surface.take();
    }

    /// the size (in pixels) of the screen we are rendering to
    pub fn size(&self) -> (u32,u32)
    {
        (self.config.width, self.config.height)
    }
}

pub struct ContextHandleInner
{
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub screen: Screen
}

impl ContextHandleInner
{
    pub(super) fn new(instance: wgpu::Instance, device: wgpu::Device, queue: wgpu::Queue) -> Self
    {
        Self 
        {
            instance,
            device,
            queue,
            screen: Screen 
            {
                surface: None,
                config: wgpu::SurfaceConfiguration
                {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: 1,
                    height: 1,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                }
            }
        }      
    }
}

#[inline]
/// gets a reference to the device instance
pub(crate) fn device(ctx: &ContextHandleInner) -> &wgpu::Device
{
    &ctx.device
}

#[inline]
/// gets a reference to the queue instance
/// 
/// A Queue executes recorded `CommandBuffer` objects and provides convenience methods 
/// for writing to buffers and textures
pub(crate) fn queue(ctx: &ContextHandleInner) -> &wgpu::Queue
{
    &ctx.queue
}
