pub mod kei;

/// 모든 state 구조체를 담을 수 있는 `StateData::extra`의 최소 크기.
///
/// xtask가 `students/src/states/**`를 보고 생성한다. 손으로 고치지 말 것.
pub const MAX_EXTRA_STATE_SIZE: usize = {
    let mut max = 0usize;
    {
        let size = ::std::mem::size_of::<kei::SubSkillState>();
        if size > max {
            max = size;
        }
    }
    max
};
