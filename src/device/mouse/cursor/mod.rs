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
