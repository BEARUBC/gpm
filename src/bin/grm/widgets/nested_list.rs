//! This module provides data structures for managing a nested, expandable list,
//! commonly used in terminal user interfaces (TUIs).
//!
//! It defines two main structs:
//! - `NestedListNode`: Represents a node in a tree-like structure. Each node can have children.
//! - `FlattenedListNode`: A "view" model representing a visible row in the final list.
//!   It contains a pre-formatted string for display and a `path` to its corresponding
//!   `NestedListNode` in the original tree.

use crate::utils;
use ratatui::text::Text;
use std::cmp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedListNode {
    pub name: String,
    /// A vector of child nodes. An empty vector indicates a leaf node.
    pub children: Vec<NestedListNode>,
    /// `true` if the node is expanded, meaning its children are visible.
    pub is_expanded: bool,
}

/// A flattened representation of a `NestedListNode` for rendering in a list widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlattenedListNode {
    /// The formatted string to be displayed in the UI. Includes indentation and an
    /// expansion indicator (e.g., '▶' or '▼').
    pub display: String,
    /// The path of indices from the root of the tree to the corresponding `NestedListNode`.
    /// For example, a path `vec![0, 2]` refers to the third child of the first root node.
    pub path: Vec<usize>,
}

/// Converts a reference to a `FlattenedListNode` into a `ratatui::text::Text` widget.
impl<'a, 'b> From<&'a FlattenedListNode> for Text<'b> {
    fn from(node: &'a FlattenedListNode) -> Self {
        // The display string is already formatted with indentation and indicators.
        // We just apply final text casing before creating the Text widget.
        let title_cased = utils::string::snake_to_title_case(&node.display);
        Text::from(title_cased)
    }
}

/// Converts a `FlattenedListNode` by value into a `ratatui::text::Text` widget.
impl<'a> From<FlattenedListNode> for Text<'a> {
    fn from(node: FlattenedListNode) -> Self {
        Text::from(&node)
    }
}

impl NestedListNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            is_expanded: false,
        }
    }

    /// Creates a new `NestedListNode` with a given name and a list of children.
    pub fn with_children(name: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            children,
            is_expanded: false,
        }
    }

    /// Recursively traverses a tree of `NestedListNode`s to create a flat list
    /// of `FlattenedListNode`s suitable for rendering.
    pub fn flatten(nodes: &[NestedListNode]) -> Vec<FlattenedListNode> {
        let mut result = Vec::new();
        let mut current_path = Vec::new();
        Self::recurse_flatten(nodes, &mut current_path, &mut result, 0);
        result
    }

    /// A private helper function for recursively building the flattened list.
    fn recurse_flatten(
        nodes: &[NestedListNode],
        path: &mut Vec<usize>,
        acc: &mut Vec<FlattenedListNode>,
        depth: usize,
    ) {
        for (i, node) in nodes.iter().enumerate() {
            path.push(i);

            let prefix = " ".repeat(depth * 2);
            let indicator = match (node.children.is_empty(), node.is_expanded) {
                (true, _) => "  ",      // Leaf node, no indicator.
                (false, true) => "▼ ",  // Expanded branch.
                (false, false) => "▶ ", // Collapsed branch.
            };

            acc.push(FlattenedListNode {
                display: format!("{}{} {}", prefix, indicator, node.name),
                // Each flattened node needs its own owned copy of the path.
                path: path.clone(),
            });

            if node.is_expanded && !node.children.is_empty() {
                Self::recurse_flatten(&node.children, path, acc, depth + 1);
            }

            path.pop();
        }
    }
}

impl FlattenedListNode {
    /// Toggles the expansion state of the corresponding `NestedListNode` in the tree.
    ///
    /// This method traverses the `tree` using the `path` stored in the `FlattenedListNode`
    /// to find and modify the correct node. It handles nested nodes correctly.
    pub fn toggle_expansion(&self, tree: &mut [NestedListNode]) {
        if self.path.is_empty() {
            return;
        }

        // The first element of the path is the index in the root `tree` slice.
        let Some((root_index, remaining_path)) = self.path.split_first() else {
            return;
        };

        // Find the top-level node.
        let Some(mut current_node) = tree.get_mut(*root_index) else {
            return; // Path is invalid, do nothing.
        };

        // Traverse down to the target node using the rest of the path.
        for &index in remaining_path {
            let Some(next_node) = current_node.children.get_mut(index) else {
                return;
            };
            current_node = next_node;
        }

        if !current_node.children.is_empty() {
            current_node.is_expanded = !current_node.is_expanded;
        }
    }

    /// Clamps a selection index to be within the valid bounds of a list.
    pub fn clamp_selection(list_len: usize, selected_index: usize) -> usize {
        if list_len == 0 {
            return 0;
        }
        cmp::min(selected_index, list_len - 1)
    }

    /// Checks if the node is at the root level of the tree.
    pub fn is_root(&self) -> bool {
        self.path.len() == 1
    }
}
