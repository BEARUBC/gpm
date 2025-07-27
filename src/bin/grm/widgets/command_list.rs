//! Renders the (nested) list of GPM commands.

use super::{
    nested_list::{FlattenedListNode, NestedListNode},
    // Ratatui's native List doesn't support nested items, so we manually flatten the tree
    // structure for rendering
    stateful_list::StatefulList,
};
use crate::client::{GpmClient, GpmResponse};
use ratatui::widgets::{List, ListItem, ListState};

pub struct CommandList {
    tree: Vec<NestedListNode>,                           // Source of truth
    pub flattened_list: StatefulList<FlattenedListNode>, // For rendering
    gpm_client: GpmClient,
}

impl CommandList {
    pub fn init() -> Self {
        let tree: Vec<NestedListNode> = gpm::enum_values!(gpm::sgcp::Resource)
            .iter()
            .map(|resource| {
                NestedListNode::with_children(
                    resource.as_str_name(),
                    gpm::get_tasks_for_resource(resource)
                        .iter()
                        .map(|task_name| NestedListNode::new(task_name))
                        .collect(),
                )
            })
            .collect();

        let flattened_list = StatefulList::with_items(NestedListNode::flatten(&tree));

        Self {
            tree,
            flattened_list,
            gpm_client: GpmClient::new(),
        }
    }

    pub fn get_list_state(&mut self) -> &mut ListState {
        &mut self.flattened_list.state
    }

    /// Handles `<enter>` clicked on a selected in the commands list. If the item is a root node
    /// (i.e a resource folder), it will be expanded. Otherwise (i.e it is a command), a request
    /// will be made to GPM with relevant parameters.
    pub fn handle_select(&mut self) -> Option<GpmResponse> {
        self.flattened_list.state.selected().and_then(|index| {
            let node = self.flattened_list.items.get_mut(index).unwrap();

            if node.is_root() {
                node.toggle_expansion(&mut self.tree);

                self.flattened_list = StatefulList::with_items(NestedListNode::flatten(&self.tree));

                self.flattened_list
                    .state
                    .select(Some(FlattenedListNode::clamp_selection(
                        self.flattened_list.items.len(),
                        index,
                    )));

                None
            } else {
                let root_index = node.path.get(0).unwrap();
                let root_node = self.tree.get(*root_index).unwrap();
                let task_code = node.path.get(1).unwrap();

                let response = self
                    .gpm_client
                    .send(
                        gpm::sgcp::Resource::from_str_name(root_node.name.as_str()).unwrap(),
                        *task_code as i32 + 1, // +1 since we don't render task 0
                    )
                    .unwrap();

                Some(response)
            }
        })
    }

    // Delegates

    pub fn next(&mut self) {
        self.flattened_list.next();
    }

    pub fn previous(&mut self) {
        self.flattened_list.previous();
    }
}

impl<'a, 'b> From<&'a CommandList> for List<'b> {
    fn from(value: &CommandList) -> Self {
        List::new(
            value
                .flattened_list
                .items
                .iter()
                .map(|node| ListItem::new(node)),
        )
    }
}
