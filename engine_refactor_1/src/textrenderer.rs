use std::ops::Range;
use std::borrow::Cow;

use crate::wgpuimpl::WGPU;
use crate::gpuprops::GPUCamera;
use crate::gpuprops::GPUSprite;
use winit::window::Window;

use glyphon::Color;
use glyphon::TextArea;
use glyphon::TextBounds;
use glyphon::Resolution;
use glyphon::Attrs;
use glyphon::Family;
use glyphon::Shaping;
use glyphon::Metrics;
use wgpu::MultisampleState;
use glyphon::Buffer;
use glyphon::TextRenderer;
use glyphon::TextAtlas;
use glyphon::SwashCache;
use glyphon::FontSystem;

pub struct TextGroup {
    text_renderer: TextRenderer,
    atlas: TextAtlas,
}

pub struct TextRenderList{
    groups: Vec<TextGroup>,
}

impl TextRenderList{

    pub fn new() -> Self {
        TextRenderList {
            groups: Vec::default(),
        }
    }

    pub fn prepare_text_render(&mut self, gpu:&WGPU, window:&Window, text: &str, x:f32, y:f32, scale:f32, color:Color) {
        // Set up text renderer
        let mut font_system = FontSystem::new();
        let mut cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&gpu.device, &gpu.queue, gpu.config.format);
        let mut text_renderer =
            TextRenderer::new(&mut atlas, &gpu.device, MultisampleState::default(), None);
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = (gpu.config.width as f64 * window.scale_factor()) as f32;
        let physical_height = (gpu.config.height as f64 * window.scale_factor()) as f32;

        buffer.set_size(&mut font_system, physical_width, physical_height);
        buffer.set_text(&mut font_system, text, Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system);
        
        text_renderer.prepare(
            &gpu.device,
            &gpu.queue,
            &mut font_system,
            &mut atlas,
            Resolution {
                width: window.inner_size().width,
                height: window.inner_size().height,
            },
            [TextArea {
                buffer: &buffer,
                left: x, 
                top: y,
                scale: scale,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 5000,
                    bottom: 5000,
                },
                default_color: color, //Color::rgb(255, 255, 255)
            }],
            &mut cache,
        ).unwrap();

        self.groups.push(TextGroup {
            text_renderer,
            atlas,
        });

    }

    pub fn clear_text_render(&mut self) {
        self.groups = Vec::default();
    }

    pub fn render<'s, 'pass>(&'s self, rpass:&mut wgpu::RenderPass<'pass>) 
        where 's: 'pass,
    {
        // rpass.set_pipeline(&self.pipeline);

        for group in self.groups.iter() {
            group.text_renderer.render(&group.atlas, rpass).unwrap();
        }
        
    }

    pub fn trim_atlas(&mut self) {

        for group in self.groups.iter_mut() {
            group.atlas.trim();
        }
        
    }
}
