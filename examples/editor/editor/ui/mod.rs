use cairo::ui::{
    tree::UIBoxTree,
    ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

static MENU_BAR_ITEMS: [&str; 6] = ["Project", "Scene", "Edit", "Debug", "Help", "About"];

static TOOLBAR_BUTTONS: [&str; 5] = ["Button 1", "Button 2", "Button 3", "Button 4", "Button 5"];

pub fn build_main_menu_bar(tree: &mut UIBoxTree) -> Result<(), String> {
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

pub fn build_toolbar(tree: &mut UIBoxTree) -> Result<(), String> {
    tree.push_parent(UIBox::new(
        "Toolbar__toolbar".to_string(),
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

    // Toolbar buttons list

    tree.push_parent(UIBox::new(
        "ToolbarItemList__toolbar_item_list".to_string(),
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

    for (button_index, button_label) in TOOLBAR_BUTTONS.iter().enumerate() {
        // Inter-item spacer.

        if button_index != 0 {
            tree.push(UIBox::new(
                "".to_string(),
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
        }

        // Button.

        let mut button_ui_box = UIBox::new(
            format!(
                "ToolbarItemList_Item{}__toolbar_item_list_item{}",
                button_index, button_index
            ),
            UIBoxFeatureFlag::DrawFill
                | UIBoxFeatureFlag::DrawBorder
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
                    size: UISize::Pixels(64),
                    strictness: 1.0,
                },
            ],
        );

        button_ui_box.text_content = Some(button_label.to_string());

        tree.push(button_ui_box)?;
    }

    tree.pop_parent()?;

    tree.pop_parent()
}

pub fn build_main_panel(tree: &mut UIBoxTree) -> Result<(), String> {
    tree.push_parent(UIBox::new(
        "Main__main".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
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

    // Main > Left (Scene, 3DViewport, Game, Assets/Console).

    tree.push_parent(UIBox::new(
        "Main_Left__main_left".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::TopToBottom,
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

    // Main > Left > Top (Outline, 3D Viewport, Game).

    tree.push_parent(UIBox::new(
        "Main_Left_Top__main_left_top".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
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

    // Main > Left > Top > Left (Outline).

    tree.push_parent(UIBox::new(
        "Main_Left_Top_Left__main_left_top_left".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::TopToBottom,
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

    let mut button_ui_box = UIBox::new(
        "Main_Left_Left_TextLabel__main_left_left_text_label".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawText,
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

    button_ui_box.text_content = Some("Outline panel".to_string());

    tree.push(button_ui_box)?;

    // Back to Main > Left > Top.

    tree.pop_parent()?;

    // Main > Left > Top > Middle (3D Viewport).

    tree.push_parent(UIBox::new(
        "Main_Left_Top_Middle__main_left_top_middle".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::TopToBottom,
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

    let mut button_ui_box = UIBox::new(
        "Main_Left_Middle_TextLabel__main_left_middle_text_label".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawText,
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

    button_ui_box.text_content = Some("3D Viewport panel".to_string());

    tree.push(button_ui_box)?;

    // Back to Main > Left > Top.

    tree.pop_parent()?;

    // Main > Left > Top > Right (Game Viewport).

    tree.push_parent(UIBox::new(
        "Main_Left_Top_Right__main_left_top_right".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::TopToBottom,
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

    let mut button_ui_box = UIBox::new(
        "Main_Left_Right_TextLabel__main_left_right_text_label".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawText,
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

    button_ui_box.text_content = Some("Game Viewport panel".to_string());

    tree.push(button_ui_box)?;

    // Back to Main > Left > Top.

    tree.pop_parent()?;

    // Back to Main > Left.

    tree.pop_parent()?;

    // Main > Left > Bottom (Assets, Console).

    tree.push_parent(UIBox::new(
        "Main_Left_Bottom__main_left_bottom".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::Pixels(325),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ],
    ))?;

    let mut button_ui_box = UIBox::new(
        "Main_Left_Bottom_TextLabel__main_left_bottom_text_label".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawText,
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

    button_ui_box.text_content = Some("Console panel".to_string());

    tree.push(button_ui_box)?;

    // Back to Main > Left.

    tree.pop_parent()?;

    // Back to Main > Left.

    tree.pop_parent()?;

    // Main > Right (Inspector).

    tree.push_parent(UIBox::new(
        "Main_Right__main_right".to_string(),
        UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::TopToBottom,
        [
            UISizeWithStrictness {
                size: UISize::Pixels(375),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ],
    ))?;

    let mut button_ui_box = UIBox::new(
        "Main_Right_TextLabel__main_right_text_label".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawText,
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

    button_ui_box.text_content = Some("Inspector panel".to_string());

    tree.push(button_ui_box)?;

    // Back to Main.

    tree.pop_parent()?;

    // Back to Root.

    tree.pop_parent()
}
