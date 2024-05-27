use cairo::ui::{
    tree::UIBoxTree,
    ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

pub fn build_editor_main_menu_bar(tree: &mut UIBoxTree) -> Result<(), String> {
    tree.push_parent(UIBox::new(
        "MainMenuBar__main_menu_bar".to_string(),
        UIBoxFeatureMask::none(),
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ],
    ))?;

    // Icon

    tree.push(UIBox::new(
        "MainMenuBarLogo__main_menu_bar_logo".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::Pixels(36),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(36),
                strictness: 1.0,
            },
        ],
    ))?;

    tree.push_parent(UIBox::new(
        "MainMenuBarMenu__main_menu_bar_menu".to_string(),
        UIBoxFeatureMask::none(),
        UILayoutDirection::TopToBottom,
        [
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::MaxOfSiblings,
                strictness: 1.0,
            },
        ],
    ))?;

    // Top spacer

    tree.push(UIBox::new(
        "MainMenuBarMenuTopSpacer__main_menu_bar_menu_top_spacer".to_string(),
        UIBoxFeatureMask::none(),
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
        ],
    ))?;

    // Menu item list

    tree.push_parent(UIBox::new(
        "MainMenuBarMenuItemList__main_menu_bar_menu_item_list".to_string(),
        UIBoxFeatureMask::none(),
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
        ],
    ))?;

    static MENU_BAR_ITEMS: [&str; 6] = ["Project", "Scene", "Edit", "Debug", "Help", "About"];

    for (item_index, item_label) in MENU_BAR_ITEMS.iter().enumerate() {
        // Inter-item spacer.

        tree.push(UIBox::new(
            "MainMenuBarMenuItemList_Spacer__menu_bar_menu_spacer".to_string(),
            UIBoxFeatureMask::none(),
            UILayoutDirection::LeftToRight,
            [
                UISizeWithStrictness {
                    size: UISize::Pixels(8),
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::MaxOfSiblings,
                    strictness: 1.0,
                },
            ],
        ))?;

        // Menu bar item (container)

        tree.push_parent(UIBox::new(
            format!(
                "MainMenuBarMenuItemList_MenuItem{}__menu_bar_menu_item_{}",
                item_index, item_index
            ),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
            UILayoutDirection::LeftToRight,
            [
                UISizeWithStrictness {
                    size: UISize::ChildrenSum,
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::ChildrenSum,
                    strictness: 1.0,
                },
            ],
        ))?;

        // Menu bar text

        let mut text_ui_box = UIBox::new(
            format!(
                "MainMenuBarMenuItemList_MenuItem{}_Text__menu_bar_menu_item_{}_text",
                item_index, item_index
            ),
            UIBoxFeatureFlag::DrawFill
                | UIBoxFeatureFlag::DrawText
                | UIBoxFeatureFlag::Hoverable
                | UIBoxFeatureFlag::Clickable,
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

        text_ui_box.text_content = Some(item_label.to_string());

        tree.push(text_ui_box)?;

        tree.pop_parent()?;
    }

    tree.pop_parent()?;

    // Bottom spacer

    tree.push(UIBox::new(
        "MainMenuBarMenuBottomSpacer__main_menu_bar_menu_bottomspacer".to_string(),
        UIBoxFeatureMask::none(),
        UILayoutDirection::LeftToRight,
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
    ))?;

    // Back to 'MainMenuBar'.

    tree.pop_parent()?;

    // Back to 'Root'.

    tree.pop_parent()
}
