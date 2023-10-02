use winit::window::Window;

#[allow(dead_code)]
pub struct WGPU {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl WGPU {
    pub async fn new(window:&Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Bump up the limits to require the availability of storage buffers.
                    limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];


        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
    
        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, size:winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    // fn render(&self, rend: impl FnMut(&mut wgpu::RenderPass, &mut wgpu::CommandEncoder)) {
    //     // ... All the 3d drawing code/render pipeline/queue/frame stuff goes here ...
    //     let frame = self.surface
    //         .get_current_texture()
    //         .expect("Failed to acquire next swap chain texture");
    //     let view = frame
    //         .texture
    //         .create_view(&wgpu::TextureViewDescriptor::default());
    //     let mut encoder =
    //         self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    //     {
    //         let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //             label: None,
    //             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //                 view: &view,
    //                 resolve_target: None,
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
    //                     store: true,
    //                 },
    //             })],
    //             depth_stencil_attachment: None,
    //         });
    //     }
        
    // }
}