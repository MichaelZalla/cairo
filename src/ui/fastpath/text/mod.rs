use crate::ui::{
    ui_box::{UIBox, UIBoxFeatureFlag, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

pub fn text(id: String, label: String) -> UIBox {
    let mut text_box = UIBox::new(
        id,
        UIBoxFeatureFlag::Null | UIBoxFeatureFlag::DrawText,
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
