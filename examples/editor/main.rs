extern crate sdl2;

use std::{cell::RefCell, env, rc::Rc};

use sdl2::mouse::Cursor;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1920_BY_1080},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color::{self, Color},
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseState},
    },
    font::cache::FontCache,
    ui::{
        context::{UIContext, GLOBAL_UI_CONTEXT},
        ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
        window::{Window, WindowList},
        UISize, UISizeWithStrictness,
    },
};

use editor::panel::EditorPanelType;

pub mod editor;

static EDITOR_UI_FILL_COLOR: Color = Color::rgb(230, 230, 230);

fn main() -> Result<(), String> {
    let title = format!("Cairo Engine (build {})", env!("GIT_COMMIT_SHORT_HASH")).to_string();

    // Initial window info.

    let mut window_info = AppWindowInfo {
        title,
        window_resolution: RESOLUTION_1920_BY_1080,
        canvas_resolution: RESOLUTION_1920_BY_1080,
        ..Default::default()
    };

    // Default render framebuffer.

    let framebuffer_rc = RefCell::new(Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    ));

    let current_frame_index_rc = RefCell::new(0_u32);

    // Initial main window.

    let window_list = {
        let mut list: WindowList<EditorPanelType> = Default::default();

        let main_window = Window {
            docked: true,
            active: true,
            focused: true,
            size: (
                window_info.window_resolution.width,
                window_info.window_resolution.height,
            ),
            ..Default::default()
        };

        // Initial panel tree.

        *main_window.panel_tree.borrow_mut() =
            editor::panel::build_main_window_panel_tree().unwrap();

        list.push(main_window);

        list
    };

    let window_list_rc = Rc::new(RefCell::new(window_list));

    // SDL will reset the cursor if we don't retain the result from
    // Cursor::set().

    let set_cursor_result_rc: RefCell<Option<Cursor>> = Default::default();

    // Primary function for rendering the UI tree to `framebuffer`; this
    // function is called when either (1) the main loop executes, or (2) the
    // user is actively resizing the main application window.

    let render_main_ui_window_to_framebuffer = |_frame_index: Option<u32>,
                                                new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        let frame_index = current_frame_index_rc.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        let mut window_list = (*window_list_rc).borrow_mut();

        if let Some(resolution) = new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);

            // Rebuild the UI tree based on the new window (root) resolution.

            for window in window_list.iter_mut() {
                if window.docked {
                    GLOBAL_UI_CONTEXT.with(|ctx| {
                        rebuild_ui_tree_for_window(
                            ctx,
                            window,
                            &resolution,
                            0.0,
                            &mut Default::default(),
                            &mut Default::default(),
                            &mut Default::default(),
                        )
                        .unwrap();
                    });
                }
            }

            // println!("Rebuilt UI tree based on the new resolution.");
        }

        framebuffer.clear(Some(color::BLACK.to_u32()));

        // Check if our application window was just resized.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            {
                // Reset cursor for this frame.

                *ctx.cursor_kind.borrow_mut() = MouseCursorKind::Arrow;
            }

            for window in window_list.iter() {
                let base_ui_tree = &mut window.ui_trees.base.borrow_mut();

                // Render the current frame's UI tree to `framebuffer`.

                base_ui_tree
                    .render_frame(*frame_index, &mut framebuffer)
                    .unwrap();
            }

            {
                let kind = &ctx.cursor_kind.borrow();

                *set_cursor_result_rc.borrow_mut() = Some(mouse::cursor::set_cursor(kind).unwrap());
            }
        });

        Ok(framebuffer.get_all().clone())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_main_ui_window_to_framebuffer);

    // Set the global font info, based on the font filepath that was passed to
    // our program.

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.font_cache
            .borrow_mut()
            .replace(FontCache::new(app.context.ttf_context));

        {
            let mut font_info = ctx.font_info.borrow_mut();

            let args: Vec<String> = env::args().collect();

            if args.len() < 2 {
                panic!("Usage: cargo run --example editor /path/to/your-font.fon");
            }

            let font_filepath = args[1].to_string();

            font_info.filepath = font_filepath;
            font_info.point_size = 14;
        }
    });

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resolution = &(*app.window_info).borrow().window_resolution;

        let mut window_list = window_list_rc.borrow_mut();

        for window in window_list.iter_mut() {
            // Rebuild the UI tree based on the latest user inputs.

            GLOBAL_UI_CONTEXT.with(|ctx| {
                rebuild_ui_tree_for_window(
                    ctx,
                    window,
                    resolution,
                    app.timing_info.seconds_since_last_update,
                    keyboard_state,
                    mouse_state,
                    game_controller_state,
                )
                .unwrap();
            });
        }

        Ok(())
    };

    let render = |frame_index: Option<u32>,
                  new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        if let Some(index) = frame_index {
            let mut current_frame_index = current_frame_index_rc.borrow_mut();

            *current_frame_index = index;
        }

        render_main_ui_window_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn rebuild_ui_tree_for_window(
    ctx: &UIContext<'static>,
    window: &mut Window<EditorPanelType>,
    resolution: &Resolution,
    seconds: f32,
    keyboard_state: &mut KeyboardState,
    mouse_state: &mut MouseState,
    game_controller_state: &mut GameControllerState,
) -> Result<(), String> {
    window.ui_trees.clear();

    {
        let ui_box_tree = &mut window.ui_trees.base.borrow_mut();

        // Bind the latest user input events.

        {
            let mut input_events = ctx.input_events.borrow_mut();

            input_events.keyboard = keyboard_state.clone();
            input_events.mouse = mouse_state.clone();
            input_events.game_controller = *game_controller_state;
        }

        // Bind delta time.

        {
            let mut seconds_since_last_update = ctx.seconds_since_last_update.borrow_mut();

            *seconds_since_last_update = seconds;
        }

        // Rebuilds the UI tree root based on the current window resolution.

        if window.docked {
            window.size.0 = resolution.width;
            window.size.1 = resolution.height;
        }

        let root_ui_box = UIBox::new(
            "Root__root".to_string(),
            UIBoxFeatureMask::none()
                | UIBoxFeatureFlag::DrawFill
                | UIBoxFeatureFlag::DrawChildDividers,
            UILayoutDirection::TopToBottom,
            [
                UISizeWithStrictness {
                    size: UISize::Pixels(window.size.0),
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::Pixels(window.size.1),
                    strictness: 1.0,
                },
            ],
        );

        ctx.fill_color(EDITOR_UI_FILL_COLOR, || {
            ui_box_tree.push_parent(root_ui_box)?;

            Ok(())
        })?;
    }

    ctx.fill_color(EDITOR_UI_FILL_COLOR, || {
        ctx.border_color(color::BLACK, || {
            let ui_box_tree = &mut window.ui_trees.base.borrow_mut();

            // Builds the main menu bar.

            editor::ui::build_main_menu_bar_ui(ctx, ui_box_tree)?;

            // Builds the toolbar.

            editor::ui::build_toolbar_ui(ctx, ui_box_tree)
        })
    })?;

    // Builds UI from the current editor panel tree.

    {
        let panel_tree = &mut window.panel_tree.borrow_mut();

        panel_tree.render(ctx, window).unwrap();
    }

    // Commit this UI tree for the current frame.

    {
        let ui_box_tree = &mut window.ui_trees.base.borrow_mut();

        ui_box_tree.commit_frame()?;
    }

    Ok(())
}
