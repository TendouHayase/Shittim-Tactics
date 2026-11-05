use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    sync::Arc,
};

use core::{
    actions::ActionContext, character::Character, simulator::Simulator, skill::Skill, state::State,
};
use error::Error;

use crate::astar::node::Node;

/// A timeline of skills and the frames they were used on, and the frame it kills the boss.
#[derive(Debug)]
pub struct Solution<'a> {
    pub timeline: Vec<(&'a dyn Skill, u16)>,
    pub kill_frame: u16,
}

/// Whether the boss is down with probability at least `threshold`.
fn is_goal(sim: &Simulator, state: &State, threshold: f64) -> bool {
    state.boss().damage_taken().tail(sim.boss.stats().hp) >= threshold
}

/// Expands in order of `g + value`, where `g` is the frame of the state.
///
/// `actions` lists the skill uses to branch on. Waiting for the next event is always tried as
/// well, so it stays a choice regardless of the policy.
pub(crate) fn best_first<'a>(
    sim: &'a Simulator,
    threshold: f64,
    value: impl Fn(&State) -> u64,
    actions: impl Fn(&State) -> Vec<ActionContext>,
) -> Result<Solution<'a>, Error> {
    let mut open: BinaryHeap<Reverse<Arc<Node>>> = BinaryHeap::new();
    // g가 항상 상태 안의 frames와 같으므로 같은 상태를 다시 펼칠 이유가 없다.
    let mut closed: HashSet<State> = HashSet::new();

    let initial = sim.initial_state();
    let h = value(&initial);
    open.push(Reverse(Arc::new(Node::new(initial, 0, h))));

    while let Some(Reverse(node)) = open.pop() {
        if is_goal(sim, &node.state, threshold) {
            return solution(sim, &node);
        }

        if !closed.insert(node.state.clone()) {
            continue;
        }

        let mut push = |state: State, action: Option<ActionContext>| {
            let g = state.frames().into();
            let h = value(&state);
            open.push(Reverse(Arc::new(Node::from_parent_node(
                state,
                g,
                h,
                node.clone(),
                action,
            ))));
        };

        for action in actions(&node.state) {
            if let Ok(state) = sim.apply(node.state.clone(), &action) {
                push(state, Some(action));
            }
        }

        if let Some(state) = wait(sim, &node.state, threshold)? {
            push(state, None);
        }
    }

    Err(Error::SearchFail(
        "no timeline reaches the threshold within the time limit",
    ))
}

/// Action X: waits for the next event, or stops early on the hit that kills the boss.
fn wait(sim: &Simulator, state: &State, threshold: f64) -> Result<Option<State>, Error> {
    let delta = sim.next_event_frames(state);
    let event_frame = (delta != u16::MAX).then(|| state.frames().saturating_add(delta));

    // 처치는 이벤트가 아니라 타격 시각에 일어나므로 타격마다 판정한다. 이벤트가 더 없어도
    // 예약된 타격으로 처치할 수 있어 끝까지 확인한다.
    let mut current = state.clone();
    while let Some(hit) = current.next_hit_frame()
        && event_frame.is_none_or(|frame| hit <= frame)
        && !sim.is_time_over(hit)
    {
        current = sim.advance(&current, hit - current.frames())?;

        if is_goal(sim, &current, threshold) {
            return Ok(Some(current));
        }
    }

    match event_frame {
        Some(frame) if !sim.is_time_over(frame) => {
            Ok(Some(sim.advance(&current, frame - current.frames())?))
        }
        _ => Ok(None),
    }
}

fn solution<'a>(sim: &'a Simulator, goal: &Arc<Node>) -> Result<Solution<'a>, Error> {
    let mut timeline = Vec::new();
    let mut node = goal.clone();

    loop {
        if let Some(action) = node.get_action() {
            let skill = sim.lookup_skill(action.skill).ok_or(Error::NotFound)?;
            timeline.push((skill, node.state.frames()));
        }

        let Some(parent) = node.get_parent() else {
            break;
        };
        node = parent;
    }

    timeline.reverse();

    Ok(Solution {
        timeline,
        kill_frame: goal.state.frames(),
    })
}
