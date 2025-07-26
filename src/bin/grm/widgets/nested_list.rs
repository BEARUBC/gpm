use ratatui::text::Text;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct FlatListItem {
    display: String,
    path: Vec<usize>,
}

impl<'a> Into<Text<'a>> for FlatListItem {
    fn into(self) -> Text<'a> {
        Text::from(self.display)
    }
}

impl TreeNode {
    fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let mut current_level = self;
        for &index in &path[..path.len() - 1] {
            current_level = current_level.children.get_mut(index)?;
        }
        current_level.children.get_mut(*path.last().unwrap())
    }

    pub fn flatten_tree(&self) -> Vec<FlatListItem> {
        let mut result: Vec<FlatListItem> = Vec::new();
        let mut path: Vec<usize> = Vec::new();

        Self::recurse_flatten(&[self.clone()], &mut path, &mut result, 0usize);

        result
    }

    fn recurse_flatten(
        nodes: &[TreeNode],
        path: &mut Vec<usize>,
        acc: &mut Vec<FlatListItem>,
        depth: usize,
    ) {
        for (i, node) in nodes.iter().enumerate() {
            path.push(i);

            let prefix = " ".repeat(depth * 2);
            let indicator = if !node.children.is_empty() {
                if node.is_expanded { "▼" } else { "▶" }
            } else {
                " "
            };

            acc.push(FlatListItem {
                display: format!("{}{} {}", prefix, indicator, node.name),
                path: path.clone(),
            });

            if node.is_expanded && !node.children.is_empty() {
                Self::recurse_flatten(&node.children, path, acc, depth + 1);
            }

            path.pop();
        }
    }
}

impl FlatListItem {
    fn toggle_expansion(&mut self, tree: &mut TreeNode, path: &[usize]) {
        if let Some(node) = tree.get_node_mut(&path) {
            if !node.children.is_empty() {
                node.is_expanded = !node.is_expanded;
            }
        }
    }

    fn clamp_selection(list: &Vec<FlatListItem>, selected_index: usize) -> usize {
        if selected_index >= list.len() {
            return list.len().saturating_sub(1);
        }
        selected_index
    }
}
