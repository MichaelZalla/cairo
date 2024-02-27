use std::cell::RefMut;

use sdl2::mouse::MouseButton;
use uuid::Uuid;

use crate::{
    buffer::Buffer2D,
    device::{GameControllerState, KeyboardState, MouseEventKind, MouseState},
    graphics::Graphics,
};

use super::{
    button::{do_button, ButtonOptions},
    context::{UIContext, UIID},
    layout::UILayoutOptions,
    layout::{item::ItemLayoutOptions, UILayoutContext, UILayoutDirection, UILayoutExtent},
    text::{do_text, TextOptions},
};

#[derive(Debug)]
pub struct PanelTitlebarOptions {
    pub title: String,
    pub closable: bool,
}

impl Default for PanelTitlebarOptions {
    fn default() -> Self {
        Self {
            title: "Panel".to_string(),
            closable: false,
        }
    }
}

static DEFAULT_CONTENT_LAYOUT_OPTIONS: UILayoutOptions = UILayoutOptions { padding: 5, gap: 8 };

#[derive(Default, Debug)]
pub struct PanelOptions {
    pub item_layout_options: ItemLayoutOptions,
    pub content_layout_options: Option<UILayoutOptions>,
    pub titlebar_options: Option<PanelTitlebarOptions>,
    pub resizable: bool,
}

#[derive(Default, Debug)]
pub struct DoPanelResult {
    pub should_close: bool,
    pub requested_resize: (i32, i32),
}

pub fn do_panel<C>(
    ctx: &mut RefMut<'_, UIContext>,
    panel_uuid: &Uuid,
    layout: &mut UILayoutContext,
    parent_buffer: &mut RefMut<'_, Buffer2D>,
    options: &PanelOptions,
    mouse_state: &MouseState,
    keyboard_state: &KeyboardState,
    game_controller_state: &GameControllerState,
    draw_children: &mut C,
) -> DoPanelResult
where
    C: FnMut(
        &mut RefMut<'_, UIContext>,
        &mut UILayoutContext,
        &Uuid,
        &UIID,
        &mut Buffer2D,
        &MouseState,
        &KeyboardState,
        &GameControllerState,
    ),
{
    let panel_id: UIID = UIID {
        item: ctx.next_id(),
    };

    if options.resizable {
        match mouse_state.button_event {
            Some(event) => match event.button {
                MouseButton::Left => match event.kind {
                    MouseEventKind::Down => {
                        let mouse_x = mouse_state.position.0;

                        if mouse_x > layout.extent.right as i32 - 4
                            && mouse_x < layout.extent.right as i32 + 4
                        {
                            ctx.set_focus_target(Some(panel_id));
                        }
                    }
                    MouseEventKind::Up => {
                        if ctx.is_focused(&panel_id) {
                            ctx.set_focus_target(None)
                        }
                    }
                },
                _ => (),
            },
            None => (),
        }
    }

    let mut requested_resize: (i32, i32) = Default::default();

    let is_resizing = ctx.is_focused(&panel_id);

    if is_resizing {
        requested_resize.0 = mouse_state.relative_motion.0;
        requested_resize.1 = mouse_state.relative_motion.1;
    }

    let should_close = draw_panel(
        ctx,
        layout,
        &panel_uuid,
        &panel_id,
        options,
        parent_buffer,
        mouse_state,
        keyboard_state,
        game_controller_state,
        draw_children,
    );

    DoPanelResult {
        should_close,
        requested_resize,
    }
}

pub static PANEL_TITLE_BAR_HEIGHT: u32 = 26;

fn draw_panel<C>(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    panel_uuid: &Uuid,
    panel_id: &UIID,
    options: &PanelOptions,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    keyboard_state: &KeyboardState,
    game_controller_state: &GameControllerState,
    draw_children: &mut C,
) -> bool
where
    C: FnMut(
        &mut RefMut<'_, UIContext>,
        &mut UILayoutContext,
        &Uuid,
        &UIID,
        &mut Buffer2D,
        &MouseState,
        &KeyboardState,
        &GameControllerState,
    ),
{
    draw_panel_frame(ctx, layout, parent_buffer);

    let mut should_close = false;

    match &options.titlebar_options {
        Some(titlebar_options) => {
            should_close =
                draw_panel_title_bar(ctx, layout, &titlebar_options, parent_buffer, mouse_state);
        }
        None => (),
    }

    let panel_contents_extent = UILayoutExtent {
        left: layout.extent.left,
        right: layout.extent.right,
        top: layout.extent.top
            + match options.titlebar_options {
                Some(_) => PANEL_TITLE_BAR_HEIGHT - 1,
                None => 0,
            },
        bottom: layout.extent.bottom,
    };

    let mut panel_contents_layout = UILayoutContext::new(
        UILayoutDirection::TopToBottom,
        panel_contents_extent,
        match options.content_layout_options {
            Some(options) => options,
            None => DEFAULT_CONTENT_LAYOUT_OPTIONS,
        },
    );

    let _ = draw_children(
        ctx,
        &mut panel_contents_layout,
        panel_uuid,
        panel_id,
        parent_buffer,
        mouse_state,
        keyboard_state,
        game_controller_state,
    );

    should_close
}

fn draw_panel_frame(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &UILayoutContext,
    parent_buffer: &mut Buffer2D,
) {
    let theme = ctx.get_theme();

    let x: u32 = layout.extent.left;
    let y = layout.extent.top;
    let width = layout.width();
    let height = layout.height();

    Graphics::rectangle(
        parent_buffer,
        x,
        y,
        width,
        height,
        Some(theme.panel_background),
        Some(theme.panel_border),
    )
}

fn draw_panel_title_bar(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    titlebar_options: &PanelTitlebarOptions,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
) -> bool {
    let theme = ctx.get_theme();

    let mut panel_titlebar_layout = UILayoutContext::new(
        UILayoutDirection::LeftToRight,
        UILayoutExtent {
            left: layout.extent.left,
            right: layout.extent.right,
            top: layout.extent.top,
            bottom: layout.extent.top + PANEL_TITLE_BAR_HEIGHT,
        },
        DEFAULT_CONTENT_LAYOUT_OPTIONS,
    );

    Graphics::rectangle(
        parent_buffer,
        layout.extent.left + 1,
        layout.extent.top + 1,
        layout.width() - 2,
        PANEL_TITLE_BAR_HEIGHT - 2,
        Some(theme.panel_titlebar_background),
        None,
    );

    let panel_titlebar_title_text_options = TextOptions {
        layout_options: ItemLayoutOptions {
            ..Default::default()
        },
        text: titlebar_options.title.clone(),
        cache: true,
        color: theme.text,
    };

    // Render the panel's title in its title bar.

    do_text(
        ctx,
        &mut panel_titlebar_layout,
        parent_buffer,
        &panel_titlebar_title_text_options,
    );

    if !titlebar_options.closable {
        return false;
    }

    static CLOSE_BUTTON_SIZE: u32 = 14;

    let panel_titlebar_close_button_options = ButtonOptions {
        layout_options: ItemLayoutOptions {
            x_offset: (panel_titlebar_layout.width()
                - (panel_titlebar_layout.get_cursor().x - panel_titlebar_layout.extent.left)
                - 40),
            ..Default::default()
        },
        label: "Close".to_string(),
        ..Default::default()
    };

    do_button(
        ctx,
        &mut panel_titlebar_layout,
        parent_buffer,
        mouse_state,
        &panel_titlebar_close_button_options,
    )
    .was_released
}
