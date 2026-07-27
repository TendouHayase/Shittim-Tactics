// === xtask gen-skills: merged from students/src/states.rs and bosses/src/states.rs, do not edit by hand ===

#[derive(Debug, Clone, Default)]
pub struct KeiState {
    /// 서브 스킬 효과로 누적된 데미지
    pub acc_damage: u64,
    /// 데미지 기록 직전까지 누적된 데미지 기록
    pub recording_start_len: usize,
}
#[derive(Debug, Clone)]
pub struct GozState {}
/// 모든 state 구조체를 담을 수 있는 `StateData::extra`의 최소 크기.
///
/// xtask가 `students/src/states.rs`와 `bosses/src/states.rs`를 보고 생성한다.
/// 손으로 고치지 말 것.
pub const MAX_EXTRA_STATE_SIZE: usize = {
    let mut max = 0usize;
    {
        let size = ::std::mem::size_of::<GozState>();
        if size > max {
            max = size;
        }
    }
    {
        let size = ::std::mem::size_of::<KeiState>();
        if size > max {
            max = size;
        }
    }
    max
};
