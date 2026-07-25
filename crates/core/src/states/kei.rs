#[derive(Debug, Clone, Default)]
pub struct SubSkillState {
    /// 서브 스킬 효과로 누적된 데미지
    pub acc_damage: u64,
    /// 데미지 기록 직전까지 누적된 데미지 기록
    pub recording_start_len: usize,
}
