use std::{cell::RefCell, rc::Rc};

use crate::{
    software_renderer::zbuffer::{self, ZBuffer},
    texture::map::{TextureMap, TextureMapWrapping},
    vec::vec3::Vec3,
};

use super::Buffer2D;

pub enum FramebufferAttachmentKind {
    Stencil,
    Depth,
    Color,
    ForwardLdr,
    ForwardOrDeferredHdr,
    Bloom,
}

#[derive(Default, Debug, Clone)]
pub struct StencilBuffer(pub Buffer2D<u8>);

impl StencilBuffer {
    pub fn set(&mut self, x: u32, y: u32) {
        self.0.set(x, y, 1);
    }
}

#[derive(Default, Debug, Clone)]
pub struct FramebufferAttachments {
    pub stencil: Option<Rc<RefCell<StencilBuffer>>>,
    pub depth: Option<Rc<RefCell<ZBuffer>>>,
    pub color: Option<Rc<RefCell<Buffer2D>>>,
    pub forward_ldr: Option<Rc<RefCell<Buffer2D>>>,
    pub deferred_hdr: Option<Rc<RefCell<Buffer2D<Vec3>>>>,
    pub bloom: Option<Rc<RefCell<TextureMap<Vec3>>>>,
}

#[derive(Default, Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub width_over_height: f32,
    pub attachments: FramebufferAttachments,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            attachments: Default::default(),
        }
    }

    pub fn complete(&mut self, projection_z_near: f32, projection_z_far: f32) {
        let (width, height) = (self.width, self.height);

        let stencil_buffer = StencilBuffer(Buffer2D::new(width, height, None));

        self.attachments.stencil = Some(Rc::new(RefCell::new(stencil_buffer)));

        let z_buffer = ZBuffer::new(width, height, projection_z_near, projection_z_far);

        self.attachments.depth = Some(Rc::new(RefCell::new(z_buffer)));

        let color_buffer = Buffer2D::new(width, height, None);

        self.attachments.color = Some(Rc::new(RefCell::new(color_buffer)));

        let forward_ldr_buffer = Buffer2D::new(width, height, None);

        self.attachments.forward_ldr = Some(Rc::new(RefCell::new(forward_ldr_buffer)));

        let deferred_hdr_buffer = Buffer2D::new(width, height, None);

        self.attachments.deferred_hdr = Some(Rc::new(RefCell::new(deferred_hdr_buffer)));

        let bloom_buffer = Buffer2D::<Vec3>::new(width, height, None);

        let mut bloom_texture_map = TextureMap::from_buffer(width, height, bloom_buffer);

        bloom_texture_map.sampling_options.wrapping = TextureMapWrapping::ClampToEdge;

        self.attachments.bloom = Some(Rc::new(RefCell::new(bloom_texture_map)));
    }

    pub fn validate(&self) -> Result<(), String> {
        let (width, height) = (self.width, self.height);

        if let Some(stencil_buffer_rc) = self.attachments.stencil.as_ref() {
            let stencil_buffer = stencil_buffer_rc.borrow();

            stencil_buffer.0.assert_dimensions(width, height);
        }

        if let Some(depth_buffer_rc) = self.attachments.depth.as_ref() {
            let depth_buffer = depth_buffer_rc.borrow();

            depth_buffer.buffer.assert_dimensions(width, height);
        }

        if let Some(color_buffer_rc) = self.attachments.color.as_ref() {
            let color_buffer = color_buffer_rc.borrow();

            color_buffer.assert_dimensions(width, height);
        }

        if let Some(forward_ldr_buffer_rc) = self.attachments.forward_ldr.as_ref() {
            let forward_ldr_buffer = forward_ldr_buffer_rc.borrow();

            forward_ldr_buffer.assert_dimensions(width, height);
        }

        if let Some(deferred_hdr_buffer_rc) = self.attachments.deferred_hdr.as_ref() {
            let deferred_hdr_buffer = deferred_hdr_buffer_rc.borrow();

            deferred_hdr_buffer.assert_dimensions(width, height);
        }

        if let Some(bloom_texture_map_rc) = self.attachments.bloom.as_ref() {
            let bloom_texture_map = bloom_texture_map_rc.borrow();

            bloom_texture_map.levels[0]
                .0
                .assert_dimensions(width, height);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        if let Some(lock) = self.attachments.stencil.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.0.clear(None);
        }

        if let Some(lock) = self.attachments.depth.as_mut() {
            let mut zbuffer = lock.borrow_mut();

            zbuffer.buffer.clear(Some(zbuffer::MAX_DEPTH));
        }

        if let Some(lock) = self.attachments.color.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.clear(None);
        }

        if let Some(lock) = self.attachments.forward_ldr.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.clear(None);
        }

        if let Some(lock) = self.attachments.deferred_hdr.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.clear(None);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, should_clear: bool) {
        self.width = width;
        self.height = height;

        self.width_over_height = self.width as f32 / self.height as f32;

        if let Some(lock) = self.attachments.stencil.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.0.resize(width, height);
            if should_clear {
                buffer.0.clear(None);
            }
        }

        if let Some(lock) = self.attachments.depth.as_mut() {
            let mut zbuffer = lock.borrow_mut();

            zbuffer.buffer.resize(width, height);

            if should_clear {
                zbuffer.buffer.clear(None);
            }
        }

        if let Some(lock) = self.attachments.color.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.resize(width, height);

            if should_clear {
                buffer.clear(None);
            }
        }

        if let Some(lock) = self.attachments.forward_ldr.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.resize(width, height);

            if should_clear {
                buffer.clear(None);
            }
        }

        if let Some(lock) = self.attachments.deferred_hdr.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.resize(width, height);

            if should_clear {
                buffer.clear(None);
            }
        }

        if let Some(lock) = self.attachments.bloom.as_mut() {
            let mut map = lock.borrow_mut();

            map.resize(width, height);
        }
    }
}
