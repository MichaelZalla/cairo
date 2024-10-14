use crate::ui::{
    ui_box::{UIBox, UIBoxFeatureFlag, UILayoutDirection},
    UISize, UISizeWithStrictness,
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
        UIBoxFeatureFlag::DrawFill
            | UIBoxFeatureFlag::DrawBorder
            | UIBoxFeatureFlag::EmbossAndDeboss
            | UIBoxFeatureFlag::DrawText
            | UIBoxFeatureFlag::Hoverable
            | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::LeftToRight,
        sizes,
        None,
    );

    button_box.text_content = Some(label);

    button_box
}
