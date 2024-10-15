use crate::ui::ui_box::tree::UIBoxTree;

use super::spacer::spacer;

pub fn stack<C, I>(
    items: &[I],
    mut callback: C,
    gap: u32,
    tree: &mut UIBoxTree,
) -> Result<(), String>
where
    C: FnMut(usize, &I, &mut UIBoxTree) -> Result<(), String>,
{
    for (index, item) in items.iter().enumerate() {
        callback(index, item, tree)?;

        if index != items.len() - 1 {
            tree.push(spacer(gap))?;
        }
    }

    Ok(())
}
