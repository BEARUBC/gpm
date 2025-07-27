use ratatui::widgets::{ListState, ScrollbarState};

#[derive(Debug)]
pub struct StatefulList<T> {
    pub items: Vec<T>,
    pub state: ListState,
    pub scrollbar_state: ScrollbarState,
}

impl<T> StatefulList<T> {
    pub fn new() -> Self {
        StatefulList {
            items: Vec::new(),
            state: ListState::default(),
            scrollbar_state: ScrollbarState::default(),
        }
    }
    pub fn with_items(items: Vec<T>) -> Self {
        let state = if items.is_empty() {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };

        Self {
            state,
            items,
            scrollbar_state: ScrollbarState::new(0),
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        self.state.select(Some(
            self.state
                .selected()
                .map_or(0, |v| (v + 1) % self.items.len()),
        ))
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        self.state.select(Some(
            self.state
                .selected()
                // Wraps the index to the end of the list if it goes < 0
                .map_or(0, |v| (v + self.items.len() - 1) % self.items.len()),
        ));
    }
}
