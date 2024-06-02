use crate::ui::{UISize, UISizeWithStrictness};

use super::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection};

pub fn container_box(
    id: String,
    layout_direction: UILayoutDirection,
    semantic_sizes: Option<[UISizeWithStrictness; 2]>,
) -> UIBox {
    let sizes = match semantic_sizes {
        Some(sizes) => sizes,
        None => [
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
        ],
    };

    UIBox::new(id, UIBoxFeatureMask::none(), layout_direction, sizes)
}

pub fn greedy_box(id: String, layout_direction: UILayoutDirection) -> UIBox {
    UIBox::new(
        id,
        UIBoxFeatureMask::none(),
        layout_direction,
        [
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ],
    )
}

pub fn text_box(id: String, label: String) -> UIBox {
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
    );

    text_box.text_content = Some(label);

    text_box
}

pub fn button_box(
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
    );

    button_box.text_content = Some(label);

    button_box
}

pub fn spacer_box(size: u32) -> UIBox {
    UIBox::new(
        "".to_string(),
        UIBoxFeatureMask::none(),
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
    )
}
