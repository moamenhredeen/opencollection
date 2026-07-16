//! Recursive traversal over collection items.

use crate::request::Item;

/// A depth-first iterator over items, descending into folders.
///
/// Yields every [`Item`] including the folders themselves; a folder is
/// yielded before its children.
#[derive(Debug, Clone)]
pub struct ItemIter<'a> {
    stack: Vec<&'a Item>,
}

impl<'a> ItemIter<'a> {
    pub(crate) fn new(items: &'a [Item]) -> Self {
        let mut stack: Vec<&'a Item> = items.iter().collect();
        stack.reverse();
        ItemIter { stack }
    }
}

impl<'a> Iterator for ItemIter<'a> {
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.stack.pop()?;
        if let Item::Folder(folder) = item
            && let Some(children) = &folder.items
        {
            self.stack.extend(children.iter().rev());
        }
        Some(item)
    }
}
