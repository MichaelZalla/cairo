use crate::ui::{
    ui_box::{UIBox, UIBoxFeatureMask, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

pub fn container(
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

    UIBox::new(id, UIBoxFeatureMask::none(), layout_direction, sizes, None)
}

pub fn greedy_container(id: String, layout_direction: UILayoutDirection) -> UIBox {
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
        None,
    )
}
