use std::cell::RefCell;

use crate::{
    pipeline::zbuffer::{self, ZBuffer},
    vec::vec3::Vec3,
};

use super::Buffer2D;

pub enum FramebufferAttachmentKind {
    Stencil,
    Depth,
    Color,
    ForwardLdr,
    ForwardOrDeferredHdr,
}

#[derive(Default, Debug)]
pub struct FramebufferAttachments {
    pub stencil: Option<RefCell<Buffer2D<u8>>>,
    pub depth: Option<RefCell<ZBuffer>>,
    pub color: Option<RefCell<Buffer2D>>,
    pub forward_ldr: Option<RefCell<Buffer2D>>,
    pub forward_or_deferred_hdr: Option<RefCell<Buffer2D<Vec3>>>,
}

#[derive(Default, Debug)]
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

    pub fn create_attachment(
        &mut self,
        kind: FramebufferAttachmentKind,
        projection_z_near: Option<f32>,
        projection_z_far: Option<f32>,
    ) {
        match kind {
            FramebufferAttachmentKind::Stencil => {
                self.attachments.stencil =
                    Some(RefCell::new(Buffer2D::new(self.width, self.height, None)));
            }
            FramebufferAttachmentKind::Depth => {
                self.attachments.depth = Some(RefCell::new(ZBuffer::new(
                    self.width,
                    self.height,
                    projection_z_near.unwrap(),
                    projection_z_far.unwrap(),
                )));
            }
            FramebufferAttachmentKind::Color => {
                self.attachments.color =
                    Some(RefCell::new(Buffer2D::new(self.width, self.height, None)));
            }
            FramebufferAttachmentKind::ForwardLdr => {
                self.attachments.forward_ldr =
                    Some(RefCell::new(Buffer2D::new(self.width, self.height, None)));
            }
            FramebufferAttachmentKind::ForwardOrDeferredHdr => {
                self.attachments.forward_or_deferred_hdr =
                    Some(RefCell::new(Buffer2D::new(self.width, self.height, None)));
            }
        }
    }

    pub fn complete(&mut self, projection_z_near: f32, projection_z_far: f32) {
        self.create_attachment(FramebufferAttachmentKind::Stencil, None, None);
        self.create_attachment(
            FramebufferAttachmentKind::Depth,
            Some(projection_z_near),
            Some(projection_z_far),
        );
        self.create_attachment(FramebufferAttachmentKind::Color, None, None);
        self.create_attachment(FramebufferAttachmentKind::ForwardLdr, None, None);
        self.create_attachment(FramebufferAttachmentKind::ForwardOrDeferredHdr, None, None);
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(lock) = self.attachments.stencil.as_ref() {
            let buffer = lock.borrow();

            assert!(buffer.width == self.width && buffer.height == self.height);
        }

        if let Some(lock) = self.attachments.depth.as_ref() {
            let zbuffer = lock.borrow();

            assert!(zbuffer.buffer.width == self.width && zbuffer.buffer.height == self.height);
        }

        if let Some(lock) = self.attachments.color.as_ref() {
            let buffer = lock.borrow();

            assert!(buffer.width == self.width && buffer.height == self.height);
        }

        if let Some(lock) = self.attachments.forward_ldr.as_ref() {
            let buffer = lock.borrow();

            assert!(buffer.width == self.width && buffer.height == self.height);
        }

        if let Some(lock) = self.attachments.forward_or_deferred_hdr.as_ref() {
            let buffer = lock.borrow();

            assert!(buffer.width == self.width && buffer.height == self.height);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        if let Some(lock) = self.attachments.stencil.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.clear(None);
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

        if let Some(lock) = self.attachments.forward_or_deferred_hdr.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.clear(None);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, should_clear: bool) {
        self.width = width;
        self.height = height;

        if let Some(lock) = self.attachments.stencil.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.resize(width, height);
            if should_clear {
                buffer.clear(None);
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

        if let Some(lock) = self.attachments.forward_or_deferred_hdr.as_mut() {
            let mut buffer = lock.borrow_mut();

            buffer.resize(width, height);

            if should_clear {
                buffer.clear(None);
            }
        }
    }
}
