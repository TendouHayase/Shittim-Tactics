#[derive(Debug, Clone)]
pub struct GozState {}

/// `StateData::extra`에 바이트째로 복사되므로 힙 포인터를 담는 타입(`Vec`, `Box`, `Arc`)을
/// 넣으면 이중 해제가 된다. 미니온을 엔티티 목록이 아니라 카운터로 추적하는 이유다.
/// `StateData::new`가 `extra`를 0으로 채우고 `Default`를 호출하지 않으므로, 모든 필드의
/// 0 비트패턴이 전투 시작 시점을 뜻해야 한다.
#[derive(Debug, Clone, Default)]
pub struct PerorodzillaState {
    /// 웨이브 소환 이후 미니온이 받은 데미지 기댓값.
    pub minion_damage: u64,

    /// 큰 미니온 한 마리의 최대 체력. `0`이면 넘어짐 판정을 건너뛴다.
    pub big_minion_hp: u64,

    /// 웨이브 소환 시점의 `boss.accumulated_damage.len()`.
    pub damage_record_start: usize,

    pub big_minions: u8,
    pub shiny_minions: u8,
    pub knocked_down: u8,
    pub small_minions: u8,

    /// 분모는 난이도별로 7 / 10 / 12다.
    pub groggy_numerator: u8,

    /// 100이면 하이퍼 스파이럴 열시선이 나간다.
    pub atg_percent: u16,
}
