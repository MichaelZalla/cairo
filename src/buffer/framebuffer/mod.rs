use std::sync::RwLock;

use crate::{
    pipeline::zbuffer::{self, ZBuffer},
    vec::vec3::Vec3,
};

use super::Buffer2D;

#[derive(Default, Debug)]
pub struct FramebufferAttachments {
    pub stencil: Option<RwLock<Buffer2D<u8>>>,
    pub depth: Option<RwLock<ZBuffer>>,
    pub color: Option<RwLock<Buffer2D>>,
    pub forward_ldr: Option<RwLock<Buffer2D>>,
    pub forward_or_deferred_hdr: Option<RwLock<Buffer2D<Vec3>>>,
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

    pub fn complete(&mut self, projection_z_near: f32, projection_z_far: f32) {
        // Bind framebuffer attachments.

        self.attachments.stencil = Some(RwLock::new(Buffer2D::new(self.width, self.height, None)));

        self.attachments.color = Some(RwLock::new(Buffer2D::new(self.width, self.height, None)));

        self.attachments.forward_ldr =
            Some(RwLock::new(Buffer2D::new(self.width, self.height, None)));

        self.attachments.forward_or_deferred_hdr =
            Some(RwLock::new(Buffer2D::new(self.width, self.height, None)));

        self.attachments.depth = Some(RwLock::new(ZBuffer::new(
            self.width,
            self.height,
            projection_z_near,
            projection_z_far,
        )));
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.attachments.stencil.as_ref() {
            Some(lock) => {
                let buffer = lock.read().unwrap();

                assert!(buffer.width == self.width && buffer.height == self.height);
            }
            None => (),
        }

        match self.attachments.depth.as_ref() {
            Some(lock) => {
                let zbuffer = lock.read().unwrap();

                assert!(zbuffer.buffer.width == self.width && zbuffer.buffer.height == self.height);
            }
            None => (),
        }

        match self.attachments.color.as_ref() {
            Some(lock) => {
                let buffer = lock.read().unwrap();

                assert!(buffer.width == self.width && buffer.height == self.height);
            }
            None => (),
        }

        match self.attachments.forward_ldr.as_ref() {
            Some(lock) => {
                let buffer = lock.read().unwrap();

                assert!(buffer.width == self.width && buffer.height == self.height);
            }
            None => (),
        }

        match self.attachments.forward_or_deferred_hdr.as_ref() {
            Some(lock) => {
                let buffer = lock.read().unwrap();

                assert!(buffer.width == self.width && buffer.height == self.height);
            }
            None => (),
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        match self.attachments.stencil.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.clear(None);
            }
            None => (),
        }

        match self.attachments.depth.as_mut() {
            Some(lock) => {
                let mut zbuffer = lock.write().unwrap();

                zbuffer.buffer.clear(Some(zbuffer::MAX_DEPTH));
            }
            None => (),
        }

        match self.attachments.color.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.clear(None);
            }
            None => (),
        }

        match self.attachments.forward_ldr.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.clear(None);
            }
            None => (),
        }

        match self.attachments.forward_or_deferred_hdr.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.clear(None);
            }
            None => (),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, should_clear: bool) {
        self.width = width;
        self.height = height;

        match self.attachments.stencil.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.resize(width, height);
                if should_clear {
                    buffer.clear(None);
                }
            }
            None => (),
        }

        match self.attachments.depth.as_mut() {
            Some(lock) => {
                let mut zbuffer = lock.write().unwrap();

                zbuffer.buffer.resize(width, height);

                if should_clear {
                    zbuffer.buffer.clear(None);
                }
            }
            None => (),
        }

        match self.attachments.color.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.resize(width, height);

                if should_clear {
                    buffer.clear(None);
                }
            }
            None => (),
        }

        match self.attachments.forward_ldr.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.resize(width, height);

                if should_clear {
                    buffer.clear(None);
                }
            }
            None => (),
        }

        match self.attachments.forward_or_deferred_hdr.as_mut() {
            Some(lock) => {
                let mut buffer = lock.write().unwrap();

                buffer.resize(width, height);

                if should_clear {
                    buffer.clear(None);
                }
            }
            None => (),
        }
    }
}
