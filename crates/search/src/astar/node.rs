use core::{actions::ActionContext, state::State};
use std::sync::Arc;

#[derive(Debug)]
pub struct Node {
    pub state: State,
    pub g: u64,
    pub f: u64,
    pub edge: Option<Edge>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    record: Arc<Node>,
    action: ActionContext,
}

impl Node {
    pub fn new(state: State, g: u64, h: u64) -> Self {
        Node {
            state,
            g,
            f: g + h,
            edge: None,
        }
    }

    pub fn from_parent_node(
        state: State,
        g: u64,
        h: u64,
        parent_node: Arc<Node>,
        action: ActionContext,
    ) -> Self {
        Node {
            state,
            g,
            f: g + h,
            edge: Some(Edge {
                record: parent_node,
                action,
            }),
        }
    }

    pub fn get_parent(&self) -> Option<Arc<Node>> {
        Some(self.edge.clone()?.record)
    }

    pub fn get_action(&self) -> Option<ActionContext> {
        Some(self.edge.clone()?.action)
    }
}

impl<'a> PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.f.cmp(&other.f)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.f.cmp(&other.f))
    }
}
