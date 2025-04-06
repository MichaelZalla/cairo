use crate::{
    buffer::Buffer2D,
    resource::handle::Handle,
    texture::{map::TextureMap, sample::TextureSamplingMethod},
    ui::{
        context::GLOBAL_UI_CONTEXT,
        extent::ScreenExtent,
        ui_box::{UIBox, UIBoxFeatureFlag, UILayoutDirection},
        UISize, UISizeWithStrictness,
    },
};

fn render_scaled_texture(
    texture: &TextureMap,
    screen_extent: &ScreenExtent,
    aspect_ratio: f32,
    target: &mut Buffer2D,
) {
    let top = screen_extent.top;
    let left = screen_extent.left;
    let width = screen_extent.right - screen_extent.left;
    let height = (width as f32 / aspect_ratio) as u32;

    texture.blit_resized(
        top,
        left,
        width,
        height,
        TextureSamplingMethod::Trilinear,
        target,
    );
}

fn image_render_callback(
    handle: &Option<Handle>,
    screen_extent: &ScreenExtent,
    target: &mut Buffer2D,
) -> Result<(), String> {
    match handle {
        Some(handle) => GLOBAL_UI_CONTEXT.with(|ctx| {
            let arena_rc_option = ctx.image_arena.borrow();

            match arena_rc_option.as_ref() {
                Some(arena_rc) => {
                    let arena = arena_rc.borrow();

                    match arena.get(handle) {
                        Ok(entry) => {
                            let texture = &entry.item;

                            match (texture.is_loaded, texture.get_aspect_ratio()) {
                                (true, Some(aspect_ratio)) => {
                                    render_scaled_texture(texture, screen_extent, aspect_ratio, target);

                                    Ok(())
                                }
                                _ => {
                                    Err("Called render_image() with a handle to an image that has not been loaded!".to_string())
                                }
                            }
                        }
                        Err(_) => Err(format!("Arena entry for Handle {} not found!", handle.uuid)),
                    }
                }
                None => Err(
                    "Called render_image() with no image arena bound to the global `UIContext`!"
                        .to_string(),
                ),
            }
        }),
        None => Err("Called render_image() with `handle` as `None`!".to_string()),
    }
}

pub fn image(
    id: String,
    handle: Handle,
    semantic_sizes: Option<[UISizeWithStrictness; 2]>,
) -> UIBox {
    let sizes = match semantic_sizes {
        Some(sizes) => sizes,
        None => [
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
        ],
    };

    UIBox::new(
        id,
        UIBoxFeatureFlag::Null | UIBoxFeatureFlag::DrawBorder,
        UILayoutDirection::LeftToRight,
        sizes,
        Some((image_render_callback, Some(handle))),
    )
}
