use crate::ui::{
    UISize, UISizeWithStrictness,
    ui_box::{UIBox, UIBoxFeatureFlags, UILayoutDirection},
};

pub fn button(
    id: String,
    label: String,
    semantic_sizes: Option<[UISizeWithStrictness; 2]>,
) -> UIBox {
    let sizes = match semantic_sizes {
        Some(sizes) => sizes,
        None => [
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
        ],
    };

    let mut button_box = UIBox::new(
        id,
        UIBoxFeatureFlags::DRAW_FILL
            | UIBoxFeatureFlags::DRAW_BORDER
            | UIBoxFeatureFlags::EMBOSS_AND_DEBOSS
            | UIBoxFeatureFlags::DRAW_TEXT
            | UIBoxFeatureFlags::HOVERABLE
            | UIBoxFeatureFlags::CLICKABLE,
        UILayoutDirection::LeftToRight,
        sizes,
        None,
    );

    button_box.text_content = Some(label);

    button_box
}
