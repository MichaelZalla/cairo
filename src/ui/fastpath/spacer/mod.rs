use crate::ui::{
    UISize, UISizeWithStrictness,
    ui_box::{UI_BOX_SPACER_ID, UIBox, UIBoxFeatureFlags, UILayoutDirection},
};

pub fn spacer(size: u32) -> UIBox {
    UIBox::new(
        UI_BOX_SPACER_ID.to_string(),
        UIBoxFeatureFlags::empty(),
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::Pixels(size),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::MaxOfSiblings,
                strictness: 1.0,
            },
        ],
        None,
    )
}

pub fn greedy_spacer() -> UIBox {
    let mut spacer = spacer(0);

    spacer.semantic_sizes[0] = UISizeWithStrictness {
        size: UISize::PercentOfParent(1.0),
        strictness: 0.0,
    };

    spacer
}
