use std::ops::Add;

// See: https://photoblogstop.com/photoshop/photoshop-blend-modes-explained/#BlendModeMath

pub enum BlendMode {
    Normal,
    Dissolve,
    Screen,
    Overlay,
    HardLight,
    SoftLight,
    ColorBurn,
    LinearBurn,
    ColorDodge,
    Add,
    Subtract,
    Multiply,
    Divide,
    VividLight,
    LinearLight,
    Darken,
    Lighten,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

pub fn blend<T: Copy + Add<Output = T>>(blend_mode: &BlendMode, lhs: &T, rhs: &T) -> T {
    match blend_mode {
        // Edits or paints each pixel to make it the result color. This is the
        // default mode. (Normal mode is called Threshold when you’re working with a
        // bitmapped or indexed-color image.)
        BlendMode::Normal => *rhs,

        // Edits or paints each pixel to make it the result color. However, the
        // result color is a random replacement of the pixels with the base color or
        // the blend color, depending on the opacity at any pixel location.
        BlendMode::Dissolve => todo!(),

        // Looks at each channel’s color information and multiplies the inverse of
        // the blend and base colors. The result color is always a lighter color.
        // Screening with black leaves the color unchanged. Screening with white
        // produces white. The effect is similar to projecting multiple photographic
        // slides on top of each other.
        BlendMode::Screen => todo!(),

        // Multiplies or screens the colors, depending on the base color. Patterns
        // or colors overlay the existing pixels while preserving the highlights and
        // shadows of the base color. The base color is not replaced, but mixed with
        // the blend color to reflect the lightness or darkness of the original
        // color.
        BlendMode::Overlay => todo!(),

        // Multiplies or screens the colors, depending on the blend color. The
        // effect is similar to shining a harsh spotlight on the image. If the blend
        // color (light source) is lighter than 50% gray, the image is lightened, as
        // if it were screened. This is useful for adding highlights to an image. If
        // the blend color is darker than 50% gray, the image is darkened, as if it
        // were multiplied. This is useful for adding shadows to an image. Painting
        // with pure black or white results in pure black or white.
        BlendMode::HardLight => todo!(),

        // Darkens or lightens the colors, depending on the blend color. The effect
        // is similar to shining a diffused spotlight on the image. If the blend
        // color (light source) is lighter than 50% gray, the image is lightened as
        // if it were dodged. If the blend color is darker than 50% gray, the image
        // is darkened as if it were burned in. Painting with pure black or white
        // produces a distinctly darker or lighter area, but does not result in pure
        // black or white.
        BlendMode::SoftLight => todo!(),

        // Looks at the color information in each channel and darkens the base color
        // to reflect the blend color by increasing the contrast between the two.
        // Blending with white produces no change.
        BlendMode::ColorBurn => todo!(),

        // Looks at the color information in each channel and darkens the base color
        // to reflect the blend color by decreasing the brightness. Blending with
        // white produces no change.
        BlendMode::LinearBurn => todo!(),

        // Looks at the color information in each channel and brightens the base
        // color to reflect the blend color by decreasing contrast between the two.
        // Blending with black produces no change.
        BlendMode::ColorDodge => todo!(),

        // Looks at the color information in each channel and brightens the base
        // color to reflect the blend color by increasing the brightness. Blending
        // with black produces no change.
        BlendMode::Add => *lhs + *rhs,

        // Looks at the color information in each channel and subtracts the blend
        // color from the base color. In 8- and 16-bit images, any resulting
        // negative values are clipped to zero.
        BlendMode::Subtract => todo!(),

        // Looks at the color information in each channel and multiplies the base
        // color by the blend color. The result color is always a darker color.
        // Multiplying any color with black produces black. Multiplying any color
        // with white leaves the color unchanged. When you’re painting with a color
        // other than black or white, successive strokes with a painting tool
        // produce progressively darker colors. The effect is similar to drawing on
        // the image with multiple marking pens.
        BlendMode::Multiply => todo!(),

        // Looks at the color information in each channel and divides the blend
        // color from the base color.
        BlendMode::Divide => todo!(),

        // Burns or dodges the colors by increasing or decreasing the contrast,
        // depending on the blend color. If the blend color (light source) is
        // lighter than 50% gray, the image is lightened by decreasing the contrast.
        // If the blend color is darker than 50% gray, the image is darkened by
        // increasing the contrast.
        BlendMode::VividLight => todo!(),

        // Burns or dodges the colors by decreasing or increasing the brightness,
        // depending on the blend color. If the blend color (light source) is
        // lighter than 50% gray, the image is lightened by increasing the
        // brightness. If the blend color is darker than 50% gray, the image is
        // darkened by decreasing the brightness.
        BlendMode::LinearLight => todo!(),

        // Looks at the color information in each channel and selects the base or
        // blend color—whichever is darker—as the result color. Pixels lighter than
        // the blend color are replaced, and pixels darker than the blend color do
        // not change.
        BlendMode::Darken => todo!(),

        // Looks at the color information in each channel and selects the base or
        // blend color—whichever is lighter—as the result color. Pixels darker than
        // the blend color are replaced, and pixels lighter than the blend color do
        // not change.
        BlendMode::Lighten => todo!(),

        // Looks at the color information in each channel and subtracts either the
        // blend color from the base color or the base color from the blend color,
        // depending on which has the greater brightness value. Blending with white
        // inverts the base color values; blending with black produces no change.
        BlendMode::Difference => todo!(),

        // Creates an effect similar to but lower in contrast than the Difference
        // mode. Blending with white inverts the base color values. Blending with
        // black produces no change.
        BlendMode::Exclusion => todo!(),

        // Creates a result color with the luminance and saturation of the base
        // color and the hue of the blend color.
        BlendMode::Hue => todo!(),

        // Creates a result color with the luminance and hue of the base color and
        // the saturation of the blend color. Painting with this mode in an area
        // with no (0) saturation (gray) causes no change.
        BlendMode::Saturation => todo!(),

        // Creates a result color with the luminance of the base color and the hue
        // and saturation of the blend color. This preserves the gray levels in the
        // image and is useful for coloring monochrome images and for tinting color
        // images.
        BlendMode::Color => todo!(),

        // Creates a result color with the hue and saturation of the base color and
        // the luminance of the blend color. This mode creates the inverse effect of
        // Color mode.
        BlendMode::Luminosity => todo!(),
    }
}
