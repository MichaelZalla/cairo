use sdl2::mouse::{Cursor, SystemCursor};

#[derive(Default, Debug, Copy, Clone)]
pub enum MouseCursorKind {
    #[default]
    Arrow,
    IBeam,
    Wait,
    DraftDiagonalRightDown,
    DragDiagonalLeftDown,
    DragLeftRight,
    DragUpDown,
    DragAll,
    No,
    Hand,
}

pub fn set_cursor(kind: &MouseCursorKind) -> Result<Cursor, String> {
    let cursor = match kind {
        MouseCursorKind::Arrow => Cursor::from_system(SystemCursor::Arrow),
        MouseCursorKind::IBeam => Cursor::from_system(SystemCursor::IBeam),
        MouseCursorKind::Wait => Cursor::from_system(SystemCursor::Wait),
        MouseCursorKind::DraftDiagonalRightDown => Cursor::from_system(SystemCursor::SizeNWSE),
        MouseCursorKind::DragDiagonalLeftDown => Cursor::from_system(SystemCursor::SizeNESW),
        MouseCursorKind::DragLeftRight => Cursor::from_system(SystemCursor::SizeWE),
        MouseCursorKind::DragUpDown => Cursor::from_system(SystemCursor::SizeNS),
        MouseCursorKind::DragAll => Cursor::from_system(SystemCursor::SizeAll),
        MouseCursorKind::No => Cursor::from_system(SystemCursor::No),
        MouseCursorKind::Hand => Cursor::from_system(SystemCursor::Hand),
    }
    .unwrap();

    cursor.set();

    Ok(cursor)
}
