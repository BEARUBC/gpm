//! This module provides a stateful list helper, a common pattern for managing
//! list widgets in `ratatui`.

use ratatui::widgets::{ListState, ScrollbarState};
use std::cmp;

/// A generic, stateful list that bundles the list items with their `ratatui` state.
#[derive(Debug)]
pub struct StatefulList<T> {
    pub items: Vec<T>,
    /// The `ratatui` state for the list, tracking the selected item.
    pub state: ListState,
    /// The `ratatui` state for a scrollbar associated with the list.
    pub scrollbar_state: ScrollbarState,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        StatefulList {
            items: Vec::new(),
            state: ListState::default(),
            scrollbar_state: ScrollbarState::default(),
        }
    }
}

impl<T> StatefulList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `StatefulList` with a given set of items.
    pub fn with_items(items: Vec<T>) -> Self {
        let item_count = items.len();
        let mut state = ListState::default();
        if item_count > 0 {
            state.select(Some(0));
        }

        Self {
            items,
            state,
            // The scrollbar's content length must match the number of items.
            scrollbar_state: ScrollbarState::new(item_count),
        }
    }

    /// Replaces the items in the list and intelligently updates the selection state.
    pub fn set_items(&mut self, items: Vec<T>) {
        let item_count = items.len();
        let previous_selection = self.state.selected();
        self.items = items;
        self.scrollbar_state = self.scrollbar_state.content_length(item_count);

        if item_count == 0 {
            self.state.select(None);
        } else if let Some(selected_index) = previous_selection {
            // Clamp the selection to the new valid range
            self.state
                .select(Some(cmp::min(selected_index, item_count - 1)));
        }
    }

    /// Moves the selection to the **next** item in the list.
    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    /// Moves the selection to the **previous** item in the list.
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + self.items.len() - 1) % self.items.len(),
            None => self.items.len() - 1,
        };
        self.state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    /// Returns a reference to the currently selected item, if any.
    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    /// Clears the selection from the list.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
