use core::{
    actions::ActionContext::{self, Use, Wait},
    skill::Skill,
    state::State,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Node<'a> {
    pub state: State,
    pub g: u64,
    pub f: u64,
    record: Option<Arc<Node<'a>>>,
    action: Option<ActionContext<'a>>,
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<'a> Eq for Node<'a> {}

impl<'a> Ord for Node<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.f.cmp(&other.f)
    }
}

impl<'a> PartialOrd for Node<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.f.cmp(&other.f))
    }
}

impl<'a> Node<'a> {
    pub fn new(state: State, g: u64, h: u64) -> Self {
        Node {
            state,
            g,
            f: g + h,
            record: None,
            action: None,
        }
    }

    pub fn from_parent_node(
        state: State,
        g: u64,
        h: u64,
        parent_node: Arc<Node<'a>>,
        action: ActionContext<'a>,
    ) -> Self {
        Node {
            state,
            g,
            f: g + h,
            record: Some(parent_node),
            action: Some(action),
        }
    }

    pub fn get_parent(&self) -> Option<Arc<Node<'a>>> {
        self.record.clone()
    }

    pub fn get_action<'b>(&'b self) -> Option<&'a dyn Skill>
    where
        'a: 'b,
    {
        match self.action.as_ref()? {
            Wait => None,
            Use(a) => Some(a.skill),
        }
    }
}
