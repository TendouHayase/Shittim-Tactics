# 객체 관계도

## Core 전체 관계 (classDiagram)

```mermaid
classDiagram

%% ═══════════════════════════════ TRAITS ═══════════════════════════

class Character {
    <<trait>>
    id() u32
    stats() BaseStats
    skill_list() Vec
}
class Stateful {
    <<trait>>
    new() Self
    students() slice
    students_mut() mut_slice
    boss() ref
    boss_mut() mut_ref
    cost() i8
    frames() u16
    is_goal() bool
    is_terminated() bool
    state_data_by_id() Option
}
class Skill {
    <<trait>>
    name() str
    owner() Weak
    cost() u8
    frames() u16
    duration() u16
    skill_mask_offset() usize
    skill_type() SkillType
    skill_effects() Vec
    apply() Vec
}
class Simulator {
    <<trait>>
    legal_actions() Vec
    apply() State
    advance() Result
    next_event_frames() u16
    damage_map() HashMap
    is_time_over() bool
    lookup_skill() Result
    character_by_id() Option
}
class Agent {
    <<trait>>
    solve() Vec
}

%% ═══════════════════════════ STRUCTS ═══════════════════════════

class BaseStats {
    level: u8
    hp: u64
    atk: u32
    def: u32
    healing: u32
    accuracy: u16
    evasion: u16
    crit: u16
    crit_res: i32
    crit_dmg: u32
    crit_dmg_res: u32
    stability: u16
    stability_rate: u16
    normal_attack_range: u16
    sighting_range: u16
    cc_power: u8
    cc_res: u8
    recovery_boost: u32
    cost_recovery: u16
    atk_speed: u32
    mov_speed: u16
    block_rate_bonus: i16
    defense_piercing: u16
    mag_count: u8
    dmg_dealt: u32
    dmg_resist: u16
    ex_skill_dmg_dealt: u32
    ex_skill_dmg_resist: u32
    basics_proficiency: u32
    healing_boost: u32
    attack_type: AttackType
    armor_type: ArmorType
    explosive_effectiveness: u32
    piercing_effectiveness: u32
    corrosive_effectiveness: u32
    mystic_effectiveness: u32
    sonic_effectiveness: u32
    buff_retention: u32
    debuff_retention: u32
}

class BossStats {
    name: String
    id: u32
    base_stats: BaseStats
    terrain: Terrain
    groggy_gauge: u64
    groggy_duration: u8
}

class Boss {
    stats: BossStats
    other_stats: T
    skills: Vec
}

class StudentSpec {
    id: u32
    name: String
    skill_levels: arr4
    weapon_level: u8
    bond_level: u8
    alter_bond_levels: Vec
    gear_tiers: arr10
    gear_levels: arr10
    talent_levels: arr3
}

class StudentStats {
    student_stats: Box
    base_stats: BaseStats
}

class Student {
    stats: StudentStats
    skills: Vec
}

class StateData {
    cooldowns: Vec
    remained_effects: BinaryHeap
    accumulated_damage: Vec
    damage_map: ref
    character: ref
    effects: SkillsBitMask
    accumulated_damage_cache: DamageCache
    coordinate: Position
    extra: byte_arr
}

class RemainedEffects {
    ticks: u16
    bit: u8
}

class AccumulatedDamage {
    ticks: u16
    damage: DamageOpt
}

class Damage {
    normal: Uniform
    crit: Uniform
    crit_num: u32
    crit_den: u32
    flags: u32
}

class SkillsBitMask {
    data: u64
}

class DamageCache {
    cached: ArcRwLock
    last_len: AtomicUsize
}

class Position {
    x: f32
    y: f32
}

class SkillEffect {
    id: tuple
    timing: EffectTiming
    targets: Vec
}

class Action {
    caster: u32
    targets: Vec
    skill: Arc
}

class TerrainCombatPower {
    street: TCPS
    outdoor: TCPS
    indoor: TCPS
}

%% ═══════════════════════════ ENUMS ═══════════════════════════

class AttackType {
    <<enum>>
    Normal Explosive Piercing Corrosive Mystic Sonic
}
class ArmorType {
    <<enum>>
    Normal Light Heavy Composite Special Elastic Structure
}
class BuffType {
    <<enum>>
    Atk Crit CritDmg Effectiveness BasicProficiency ExSkillDmgDealt DmgDealt Def CostRecovery
}
class DebuffType {
    <<enum>>
    Atk Crit CritDmg Effectiveness ExSkillDmgDealt BasicProficiency DmgDealt Def CostRecovery
}
class EffectKind {
    <<enum>>
    Damage Heal Buff Debuff Move Other
}
class EffectTiming {
    <<enum>>
    Instant Persistent
}
class SkillEffectTarget {
    <<enum>>
    Boss Student Land Oneself
}
class Region {
    <<enum>>
    Polygon Arc
}
class SkillType {
    <<enum>>
    Ex Basic Enhanced Sub NormalAttack
}
class Difficulty {
    <<enum>>
    Normal Hard VeryHard Hardcore Extreme Insane Torment Lunatic
}
class Terrain {
    <<enum>>
    Street Outdoor Indoor
}
class TCPS {
    <<enum>>
    SS S A B C D
}
class ActionContext {
    <<enum>>
    Wait Use
}

%% ═════════════════════ MEMBER FIELDS (owns) ═══════════════════

BossStats ..|> BaseStats : "base_stats"
Boss ..|> BossStats : "stats"
Boss ..|> Skill : "skills[]"
Boss ..|> Character : "impl"

StudentSpec ..|> BaseStats : "(via StudentStats)"
StudentStats ..|> StudentSpec : "student_stats"
StudentStats ..|> BaseStats : "base_stats"
Student ..|> StudentStats : "stats"
Student ..|> Skill : "skills[]"
Student ..|> Character : "impl"

StateData ..|> RemainedEffects : "remained_effects[]"
StateData ..|> AccumulatedDamage : "accumulated_damage[]"
StateData ..|> DamageCache : "accumulated_damage_cache"
StateData ..|> SkillsBitMask : "effects damage_map"
StateData ..|> Position : "coordinate"
StateData ..|> Character : "character"
AccumulatedDamage ..|> Damage : "damage"

SkillEffect ..|> EffectTiming : "timing"
SkillEffect ..|> SkillEffectTarget : "targets[]"
SkillEffectTarget ..|> EffectKind : "kind"
SkillEffectTarget ..|> Region : "region(for Land)"
EffectKind ..|> BuffType : "Buff variant"
EffectKind ..|> DebuffType : "Debuff variant"

Damage ..|> stochastic_Uniform : "normal crit"
DamageCache ..|> stochastic_IrwinHall : "cached"

ActionContext ..|> Action : "Use variant"
Action ..|> Skill : "skill"

BaseStats ..|> AttackType : "attack_type"
BaseStats ..|> ArmorType : "armor_type"
BossStats ..|> Terrain : "terrain"
TerrainCombatPower ..|> TCPS : "street outdoor indoor"

Stateful ..|> StateData : "returns"
Simulator ..|> Stateful : "generic S"
Simulator ..|> Skill : "returns actions"
Agent ..|> Skill : "returns"
Agent ..|> Stateful : "generic S"
```

---

## Crate Dependency Graph

```mermaid
flowchart LR
    stochastic --> core
    error --> core
    macros --> core
    parsing["parsing-json"] --> bosses
    error --> parsing

    core --> bosses
    core --> students
    core --> simulator
    core --> search
    core --> solver

    bosses --> simulator
    bosses --> search
    students --> simulator

    simulator --> solver
    search --> solver
```

---

## 아키텍처 노트

| 항목 | 설명 |
|------|------|
| `StateData.extra` | `const EXTRA_BYTES` 제네릭, `[u8; N]` inline. `extra_as::<T>()` / `extra_as_mut::<T>()` 접근 |
| `create_state!` | `(name, boss_extra, students...)` → MAX = max(size_of) → State + impl Stateful |
| `Skill(T=())` | `SubSkill`은 `T=SubSkillState` → `dyn Skill`에 못 담음 |
| `EffectKind::Other` | `fn(&T, S) -> S` 함수 포인터. concrete type 필요 |
| `Simulation(T,N,S)` | S = create_state! 생성 타입. tick 기반 advance/apply |
| A* | `BinaryHeap<Reverse<Arc<Node>>>` open, `HashMap<S, u64>` closed |
| Solver | `Simulator` + `Algorithm` → `Agent` |
| Toolchain | `stable-x86_64-pc-windows-msvc` 필수 (gnu GUI crash) |
| UI | `eframe/egui 0.31`. Config/Results 탭. solver stub |
