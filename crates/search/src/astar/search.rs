use crate::{
    algorithm::Algorithm,
    astar::{heuristics::heuristics, node::Node},
};
use core::{
    skill::Skill,
    state::{State, Stateful},
};
use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

pub struct Astar<const N: usize> {}

impl<const N: usize> Algorithm for Astar<N> {
    type S<'a> = State<'a, N>;

    fn search<'a>(
        &self,
        simulator: &impl core::simulator::Simulator,
        initial: Self::S<'a>,
    ) -> Vec<std::sync::Arc<dyn core::skill::Skill>> {
        let mut result: Vec<Node<'_, State<'a, N>>> = vec![];

        let mut pq: BinaryHeap<Reverse<Node<'_, State<'_, N>>>> = BinaryHeap::new();

        let init_h = heuristics(simulator, &initial);

        let init_node = Node::new(initial, 0, init_h);

        pq.push(Reverse(init_node));

        while !pq.is_empty() {
            let node = pq.pop().unwrap();

            if node.0.state.is_goal() {
                break;
            }
        }

        result
    }
}
