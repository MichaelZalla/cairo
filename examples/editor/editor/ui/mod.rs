use cairo::ui::{
    context::UIContext,
    fastpath::{
        button::button,
        container::{container, greedy_container},
        spacer::spacer,
    },
    ui_box::{tree::UIBoxTree, UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

static MENU_BAR_ITEMS: [&str; 8] = [
    "File", "Edit", "Select", "View", "Scene", "Render", "Window", "Help",
];

static TOOLBAR_BUTTONS: [&str; 5] = ["Button 1", "Button 2", "Button 3", "Button 4", "Button 5"];

pub fn build_main_menu_bar_ui(_ctx: &UIContext, tree: &mut UIBoxTree) -> Result<(), String> {
    tree.push_parent(container(
        "MainMenuBar".to_string(),
        UILayoutDirection::LeftToRight,
        Some([
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ]),
    ))?;

    // Icon

    tree.push(UIBox::new(
        "MainMenuBarLogo".to_string(),
        UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::Pixels(24),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(24),
                strictness: 1.0,
            },
        ],
        None,
    ))?;

    tree.push_parent(container(
        "MainMenuBarMenu".to_string(),
        UILayoutDirection::TopToBottom,
        Some([
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::MaxOfSiblings,
                strictness: 1.0,
            },
        ]),
    ))?;

    // Top spacer

    tree.push(greedy_container(
        "MainMenuBarMenuTopSpacer".to_string(),
        UILayoutDirection::LeftToRight,
    ))?;

    // Menu item list

    tree.push_parent(container(
        "MainMenuBarMenuItemList".to_string(),
        UILayoutDirection::LeftToRight,
        None,
    ))?;

    for (item_index, item_label) in MENU_BAR_ITEMS.iter().enumerate() {
        // Inter-item spacer.

        tree.push(spacer(10))?;

        // Menu bar item (container)

        tree.push_parent(container(
            format!("MainMenuBarMenuItemList_MenuItem{}", item_index),
            UILayoutDirection::LeftToRight,
            None,
        ))?;

        // Menu bar button (text)

        let mut button_box = button(
            format!("MainMenuBarMenuItemList_MenuItem{}_Text", item_index),
            item_label.to_string(),
            None,
        );

        button_box.features ^= UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::EmbossAndDeboss;

        tree.push(button_box)?;

        tree.pop_parent()?;
    }

    tree.pop_parent()?;

    // Bottom spacer

    tree.push(greedy_container(
        "MainMenuBarMenuBottomSpacer".to_string(),
        UILayoutDirection::LeftToRight,
    ))?;

    // Back to 'MainMenuBar'.

    tree.pop_parent()?;

    // Back to 'Root'.

    tree.pop_parent()
}

pub fn build_toolbar_ui(_ctx: &UIContext, tree: &mut UIBoxTree) -> Result<(), String> {
    tree.push_parent(container(
        "Toolbar".to_string(),
        UILayoutDirection::LeftToRight,
        Some([
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ]),
    ))?;

    // Toolbar buttons list

    tree.push_parent(container(
        "ToolbarItemList".to_string(),
        UILayoutDirection::LeftToRight,
        None,
    ))?;

    for (button_index, button_label) in TOOLBAR_BUTTONS.iter().enumerate() {
        // Inter-item spacer.

        if button_index != 0 {
            tree.push(spacer(8))?;
        }

        // Button.

        tree.push(button(
            format!("ToolbarItemList_Item{}", button_index),
            button_label.to_string(),
            Some([
                UISizeWithStrictness {
                    size: UISize::TextContent,
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::Pixels(45),
                    strictness: 1.0,
                },
            ]),
        ))?;
    }

    tree.pop_parent()?;

    tree.pop_parent()
}
