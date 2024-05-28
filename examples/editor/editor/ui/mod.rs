use cairo::ui::{
    context::UIContext,
    ui_box::{
        utils::{button_box, container_box, greedy_box, spacer_box, text_box},
        UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

static MENU_BAR_ITEMS: [&str; 6] = ["Project", "Scene", "Edit", "Debug", "Help", "About"];

static TOOLBAR_BUTTONS: [&str; 5] = ["Button 1", "Button 2", "Button 3", "Button 4", "Button 5"];

pub fn build_main_menu_bar_ui(ctx: &UIContext) -> Result<(), String> {
    let mut tree = ctx.tree.borrow_mut();

    tree.push_parent(container_box(
        "MainMenuBar__main_menu_bar".to_string(),
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
        "MainMenuBarLogo__main_menu_bar_logo".to_string(),
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
    ))?;

    tree.push_parent(container_box(
        "MainMenuBarMenu__main_menu_bar_menu".to_string(),
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

    tree.push(greedy_box(
        "MainMenuBarMenuTopSpacer__main_menu_bar_menu_top_spacer".to_string(),
        UILayoutDirection::LeftToRight,
    ))?;

    // Menu item list

    tree.push_parent(container_box(
        "MainMenuBarMenuItemList__main_menu_bar_menu_item_list".to_string(),
        UILayoutDirection::LeftToRight,
        None,
    ))?;

    for (item_index, item_label) in MENU_BAR_ITEMS.iter().enumerate() {
        // Inter-item spacer.

        tree.push(spacer_box(8))?;

        // Menu bar item (container)

        tree.push_parent(container_box(
            format!(
                "MainMenuBarMenuItemList_MenuItem{}__menu_bar_menu_item_{}",
                item_index, item_index
            ),
            UILayoutDirection::LeftToRight,
            None,
        ))?;

        // Menu bar text

        tree.push(text_box(
            format!(
                "MainMenuBarMenuItemList_MenuItem{}_Text__menu_bar_menu_item_{}_text",
                item_index, item_index
            ),
            item_label.to_string(),
        ))?;

        tree.pop_parent()?;
    }

    tree.pop_parent()?;

    // Bottom spacer

    tree.push(greedy_box(
        "MainMenuBarMenuBottomSpacer__main_menu_bar_menu_bottomspacer".to_string(),
        UILayoutDirection::LeftToRight,
    ))?;

    // Back to 'MainMenuBar'.

    tree.pop_parent()?;

    // Back to 'Root'.

    tree.pop_parent()
}

pub fn build_toolbar_ui(ctx: &UIContext) -> Result<(), String> {
    let mut tree = ctx.tree.borrow_mut();

    tree.push_parent(container_box(
        "Toolbar__toolbar".to_string(),
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

    tree.push_parent(container_box(
        "ToolbarItemList__toolbar_item_list".to_string(),
        UILayoutDirection::LeftToRight,
        None,
    ))?;

    for (button_index, button_label) in TOOLBAR_BUTTONS.iter().enumerate() {
        // Inter-item spacer.

        if button_index != 0 {
            tree.push(spacer_box(8))?;
        }

        // Button.

        tree.push(button_box(
            format!(
                "ToolbarItemList_Item{}__toolbar_item_list_item{}",
                button_index, button_index
            ),
            button_label.to_string(),
            [
                UISizeWithStrictness {
                    size: UISize::TextContent,
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::Pixels(45),
                    strictness: 1.0,
                },
            ],
        ))?;
    }

    tree.pop_parent()?;

    tree.pop_parent()
}
