use ratatui::text::Text;

#[derive(Debug, Clone)]
pub struct NestedListNode {
    pub name: String,
    pub children: Vec<NestedListNode>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct FlattenedListNode {
    pub display: String,
    pub path: Vec<usize>,
}

impl<'a> Into<Text<'a>> for &'a FlattenedListNode {
    fn into(self) -> Text<'a> {
        Text::from(self.display.as_str())
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

    pub fn with_children(name: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            children,
            is_expanded: false,
        }
    }

    fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut NestedListNode> {
        if path.is_empty() {
            return None;
        }
        let mut current_level = self;
        if path.len() == 1 {
            current_level.children.get_mut(*path.get(0).unwrap())
        } else {
            for &index in &path[..path.len() - 1] {
                current_level = current_level.children.get_mut(index)?;
            }
            current_level.children.get_mut(*path.last().unwrap())
        }
    }

    pub fn flatten(nodes: &Vec<NestedListNode>) -> Vec<FlattenedListNode> {
        let mut result: Vec<FlattenedListNode> = Vec::new();
        let mut path: Vec<usize> = Vec::new();

        Self::recurse_flatten(nodes, &mut path, &mut result, 0usize);

        result
    }

    fn recurse_flatten(
        nodes: &[NestedListNode],
        path: &mut Vec<usize>,
        acc: &mut Vec<FlattenedListNode>,
        depth: usize,
    ) {
        for (i, node) in nodes.iter().enumerate() {
            path.push(i);

            let prefix = " ".repeat(depth);
            let indicator = if !node.children.is_empty() {
                if node.is_expanded { "▼" } else { "▶" }
            } else {
                " "
            };

            acc.push(FlattenedListNode {
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

// TODO: Could use typestate pattern to manage the sync between list and tree view?

// TODO: improve
impl FlattenedListNode {
    pub fn toggle_expansion(&mut self, tree: &mut Vec<NestedListNode>) {
        for (i, tree_node) in tree.iter_mut().enumerate() {
            if *self.path.get(0).unwrap() == i {
                if !tree_node.children.is_empty() {
                    tree_node.is_expanded = !tree_node.is_expanded;
                }
            }
        }
    }

    pub fn clamp_selection(list_len: usize, selected_index: usize) -> usize {
        if selected_index >= list_len {
            return list_len.saturating_sub(1);
        }
        selected_index
    }

    pub fn is_root(&self) -> bool {
        self.path.len() == 1
    }
}
