use core::{
    actions::ActionContext::{self, Use, Wait},
    skill::Skill,
    state::Stateful,
};
use std::{marker::PhantomData, sync::Arc};

#[derive(Debug)]
pub struct Node<'a, S: Stateful<'a>> {
    pub state: S,
    pub g: u64,
    pub f: u64,
    record: Option<Arc<Node<'a, S>>>,
    action: Option<ActionContext<dyn Skill>>,
    _marker: PhantomData<&'a S>,
}

impl<'a, S: Stateful<'a> + Eq> PartialEq for Node<'a, S> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<'a, S: Stateful<'a> + Eq> Eq for Node<'a, S> {}

impl<'a, S: Stateful<'a> + Eq> Ord for Node<'a, S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.f.cmp(&other.f)
    }
}

impl<'a, S: Stateful<'a> + Eq> PartialOrd for Node<'a, S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.f.cmp(&other.f))
    }
}

impl<'a, S: Stateful<'a>> Node<'a, S> {
    pub fn new(state: S, g: u64, h: u64) -> Self {
        Node {
            state,
            g,
            f: g + h,
            record: None,
            action: None,
            _marker: PhantomData,
        }
    }

    pub fn from_parent_node(
        state: S,
        g: u64,
        h: u64,
        parent_node: Arc<Node<'a, S>>,
        action: ActionContext<dyn Skill>,
    ) -> Self {
        Node {
            state,
            g,
            f: g + h,
            record: Some(parent_node),
            action: Some(action),
            _marker: PhantomData,
        }
    }

    pub fn get_parent(&self) -> Option<Arc<Node<'a, S>>> {
        self.record.clone()
    }

    pub fn get_action(&self) -> Option<Arc<dyn Skill>> {
        match self.action.as_ref()? {
            Wait => None,
            Use(a) => Some(a.skill.clone()),
        }
    }
}
