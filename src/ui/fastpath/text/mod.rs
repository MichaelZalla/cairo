use crate::ui::{
    UISize, UISizeWithStrictness,
    ui_box::{UIBox, UIBoxFeatureFlags, UILayoutDirection},
};

pub fn text(id: String, label: String) -> UIBox {
    let mut text_box = UIBox::new(
        id,
        UIBoxFeatureFlags::empty() | UIBoxFeatureFlags::DRAW_TEXT,
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
        ],
        None,
    );

    text_box.text_content = Some(label);

    text_box
}
