# 데이터 파일 포맷

보스와 학생의 수치는 코드가 아니라 `data/` 아래 JSON에 있습니다. 밸런스 패치가 오면
Rust를 건드리지 않고 이 파일들만 고칩니다.

```
data/
├── bosses/
│   ├── binah.json
│   ├── goz.json
│   └── perorodzilla.json
└── students/
    └── list.json
```

---

## 먼저 알아둘 것

### 값이 빠지면 파싱이 실패합니다

기본값으로 대신 채워주지 않습니다. 필드 하나만 빠져도 그 파일 전체를 못 읽습니다.
아직 모르는 값이라면 `0`을 저장합니다.

### enum은 전부 소문자입니다

Rust 쪽 이름(`Heavy`)이 아니라 JSON 쪽 이름(`heavy`)을 씁니다.

| 종류      | 쓸 수 있는 값                                                                |
| --------- | ---------------------------------------------------------------------------- |
| 공격 타입 | `normal` `explosive` `piercing` `corrosive` `mystic` `sonic`                 |
| 방어 타입 | `normal` `light` `heavy` `composite` `special` `elastic` `structure`         |
| 난이도    | `normal` `hard` `veryhard` `hardcore` `extreme` `insane` `torment` `lunatic` |

### 숫자 단위가 두 가지입니다

| 어디      | 단위                               | 예                         |
| --------- | ---------------------------------- | -------------------------- |
| 스탯      | 만분율                             | `10000` = 100%, `200` = 2% |
| 스킬 계수 | 백분율                             | `100` = 100%, `750` = 750% |
| 시간      | 틱(30틱 = 1초)                     | `90` = 3초                 |
| 좌표      | 정수 `[x, y]`, 보스 기준 상대 위치 | `[-150, 2200]`             |

### 순수 JSON입니다

주석(`//`)과 마지막 쉼표를 쓸 수 없습니다. 이 문서의 예시에 붙은 설명은 전부 코드 블록
바깥에 있습니다.

### id는 다음과 같은 규칙으로 만들어집니다

| 비트범위 | 설명                                                    |
| -------: | :------------------------------------------------------ |
|    31:31 | 보스, 학생 구분. 보스는 1, 학생은 0                     |
|     0:30 | 보스의 출시 순 번호(~9999), 학생의 경우 고유 ID(10000~) |

---

## 보스 파일

`data/bosses/<보스>.json`. 큰 틀은 네 덩어리입니다.

```json
{
    "id": 0,
    "name": { "ko": "비나", "ja": "ビナー", "en": "Binah" },
    "heavy": { "normal": { }, "lunatic": { } },
    "skills": { "AtsilutsLight": { } }
}
```

| 키        | 뜻                                                                                           |
| --------- | -------------------------------------------------------------------------------------------- |
| `id`      | 게임이 정한 보스 번호. 난이도마다 다른 값이 아니라 파일에 한 번만 옵니다.                    |
| `name`    | 표시 이름. 세 언어를 전부 채웁니다.                                                          |
| 방어 타입 | 그 방어 타입일 때의 난이도별 스탯. 방어 타입이 난이도마다 바뀌는 보스는 키를 여러 개 둡니다. |
| `skills`  | 스킬별 수치. 방어 타입과 무관해서 바깥에 나와 있습니다.                                      |

`id` `name` `skills`를 뺀 나머지 최상위 키는 전부 방어 타입으로 읽습니다. 오타를 내면
"그런 방어 타입 없음"으로 실패합니다.

표시 이름은 시뮬레이션 시작 전 언어가 한 번 정해지고(`core::locale::set_language`) 그 언어만
읽힙니다. 스킬의 `name`도 같습니다.

### 난이도 하나의 스탯

방어 타입 → 난이도 아래에 스탯이 들어갑니다. 따로 감싸는 객체가 없습니다.

```json
"heavy": {
    "lunatic": {
        "level": 0,
        "hp": 50000000,
        "atk": 30000,
        "def": 7200,
        "healing": 9000,

        "accuracy": 1300,
        "evasion": 100,
        "crit": 200,
        "crit_res": 20,
        "crit_dmg": 20000,
        "crit_dmg_res": 8000,
        "stability": 300,
        "stability_rate": 7500,

        "normal_attack_range": 3000,
        "sighting_range": 65535,
        "mov_speed": 500,
        "atk_speed": 10000,
        "mag_count": 1,

        "cc_power": 100,
        "cc_res": 100,
        "cost_recovery": 700,
        "recovery_boost": 10000,
        "healing_boost": 10000,

        "block_rate_bonus": 0,
        "defense_piercing": 0,
        "dmg_dealt": 10000,
        "dmg_resist": 10000,
        "ex_skill_dmg_dealt": 10000,
        "ex_skill_dmg_resist": 10000,
        "basics_proficiency": 10000,
        "buff_retention": 10000,
        "debuff_retention": 10000,

        "attack_type": "normal",
        "armor_type": "heavy",
        "explosive_effectiveness": 10000,
        "piercing_effectiveness": 10000,
        "corrosive_effectiveness": 10000,
        "mystic_effectiveness": 10000,
        "sonic_effectiveness": 10000,

        "groggy_gauge": 10000000,
        "groggy_duration": 10,
        "phase_switching_hp": [30000000, 12500000, 0]
    }
}
```

`phase_switching_hp`는 길이가 반드시 **3**입니다. 페이즈 전환 지점이 그보다 적으면 남는
자리에 `0`을 넣습니다. 위 예시는 3페이즈 보스라 전환점이 둘뿐입니다.

### skills

```json
"skills": {
    "PurifyingStorm": {
        "name": { "ko": "정화의 폭풍", "ja": "", "en": "Purifying Storm" },
        "cost":              [3, 3, 3, 3, 3, 3, 3, 3],
        "duration":          [30, 30, 30, 30, 30, 30, 30, 30],
        "frames":            [0, 0, 0, 0, 0, 0, 0, 0],
        "percent":           [300, 300, 300, 300, 300, 300, 300, 300],
        "def_down_scale":    [50, 50, 50, 50, 50, 50, 50, 50],
        "def_down_duration": [90, 90, 90, 90, 90, 90, 90, 90],
        "count":             [4, 4, 4, 4, 4, 4, 4, 4]
    }
}
```

- **키는 스킬 구조체 이름**입니다. `crates/bosses/src/<보스>/skills.rs`에 있는 이름에서
  보스 접두사를 뺀 것입니다. 예: `PurifyingStorm`.
- **`name`을 뺀 필드 이름은 그 스킬의 `params` 구조체 필드와 1:1로 맞습니다.** 새 수치를
  넣으려면 Rust 쪽 필드를 먼저 추가해야 합니다.
- **모든 수치는 길이 8짜리 배열이고 난이도순입니다.**
- 새 난이도 추가시 길이가 늘어납니다.

배열 자리는 항상 이 순서입니다.

| 0      | 1    | 2        | 3        | 4       | 5      | 6       | 7       |
| ------ | ---- | -------- | -------- | ------- | ------ | ------- | ------- |
| normal | hard | veryhard | hardcore | extreme | insane | torment | lunatic |

난이도와 무관한 수치라도 같은 값을 여덟 번 적습니다. 중간을 빼먹으면 뒤가 전부 한 칸씩
밀려서, 조용히 틀린 값으로 계산됩니다.

난이도에 따라 달라지는 수치는 이렇게 보입니다. 토먼트부터 계수가 오르는 경우입니다.

```json
"instant_percent": [120, 120, 120, 120, 120, 120, 160, 200]
```

수치 자체가 원래 배열이어도 **0번째 축은 항상 난이도**입니다. 아래는 거리순 4명에게 각각
들어가는 계수가 인세인부터 두 배가 되는 경우입니다.

```json
"nearest_percents": [
    [375,150,150,75], [375,150,150,75], [375,150,150,75], [375,150,150,75],
    [375,150,150,75], [750,300,300,150], [750,300,300,150], [750,300,300,150]
]
```

값이 아직 없는 자리는 `null`입니다. 그 값을 쓰는 효과는 조용히 빠집니다.

```json
"blast_region": [null, null, null, null, null, null, null, null]
```

### 범위

스킬이 범위를 가지면 `shape`로 종류를 구분합니다.

사각형·삼각형은 `polygon`입니다.

```json
{
    "shape": "polygon",
    "vertex": [[-150, 2200], [150, 2200], [150, 0], [-150, 0]],
    "count": 4
}
```

`vertex`는 **항상 4개를 채우고**, 실제로 쓰는 개수만 `count`에 적습니다. 삼각형이면
`count`가 3이고 네 번째 정점은 무시되므로 `[0, 0]`을 넣어두면 됩니다.

부채꼴은 `arc`입니다. 각도는 도(degree) 단위입니다.

```json
{
    "shape": "arc",
    "radius": 800,
    "start_angle_degree": 0,
    "end_angle_degree": 90
}
```

---

## 학생 파일

`data/students/<학생>.json`. 어떤 학생이 있는지는 `data/students/list.json`에 이름만
나열합니다.

**레벨·성급·성작·능력개방으로 오르는 수치는 이 파일에 없습니다.** 전부 공용 수식이고,
그 수식은 1성 90레벨 값에서 출발합니다. 여기 적는 것은 그 수식의 입력이 되는 학생
고유값뿐입니다.

```json
{
    "id": 10135,
    "name": { "ko": "텐도 케이", "ja": "天童 ケイ", "en": "Tendou Kei" },
    "terrain_adaptation": { "street": "s", "outdoor": "d", "indoor": "a" },
    "gear_slots": ["hat", "wristwatch", "necklace"],
    "lvl1_stats": { },
    "delta": { },
    "stats_at_90": { },
    "unique_weapon": { },
    "skills": { }
}
```

| 키 | 뜻 |
| --- | --- |
| `id` | 게임이 정한 학생 번호. |
| `name` | 표시 이름. 세 언어를 전부 채웁니다. |
| `terrain_adaptation` | 시가지·야외·실내 지형적성. `ss` `s` `a` `b` `c` `d`. |
| `gear_slots` | 낄 수 있는 장비 3종. 수치는 여기 없고 장비 데이터에 있습니다. |
| `lvl1_stats` | 1레벨 스탯. 보스의 난이도별 스탯과 같은 필드 구성입니다. |
| `delta` | 레벨 1당 증가량. |
| `stats_at_90` | 90레벨 스탯. 레벨을 타는 네 스탯만 적습니다. |
| `unique_weapon` | 전용무기 1~4성. |
| `skills` | 스킬별 수치. |

`lvl1_stats`에는 **`level`을 적지 않습니다.** 레벨은 데이터가 아니라 편성에서 정해지는
런타임 값이라, 이 파일에서 읽는 것은 1레벨 값뿐이고 실제 레벨은 나중에 채워집니다. 보스
파일은 난이도마다 레벨이 고정이라 그쪽에는 `level`이 있습니다.

장비 종류는 `hat` `gloves` `shoes` `bag` `badge` `hairpin` `amulet` `wristwatch`
`necklace` `unique_item`입니다.

### 소수를 그대로 적습니다

스탯 증가량과 스킬 버프 배율은 만분율로 접지 않고 게임이 표시하는 소수를 그대로 씁니다.

```json
"delta": { "hp": 149, "atk": 41, "def": 3.5, "healing": 26.8 }
```

읽을 때 반올림하지 않고 분수(`35/10`)로 보관해서, 곱셈을 먼저 하고 나눗셈을 마지막에
합니다. 스탯 몇십 차이로 택틱 성패가 갈리기 때문에 여기서 부동소수로 접으면 안 됩니다.

소수 자릿수는 적힌 그대로 살아납니다. `26.80`이라고 적어도 `26.8`과 같은 값이 되지만,
유효숫자 17자리를 넘기면 복원되지 않습니다.

### unique_weapon

```json
"unique_weapon": {
    "hp": [0, 0, 0, 0],
    "atk": [0, 0, 0, 0],
    "enhanced_skill_plus": {
        "buff": {
            "crit_dmg": { "amount": [], "scale": [] }
        }
    },
    "effectiveness_bonus": 1000
}
```

`hp`와 `atk`는 **1성부터 4성까지** 넷입니다. 0성(미장착)은 배열에 없으므로 색인이
`성급 - 1`입니다.

성급마다 붙는 효과는 이렇습니다.

| 성급 | 효과 | 어디에 적나 |
| --- | --- | --- |
| 1 | 스탯 증가 | `hp[0]`, `atk[0]` |
| 2 | 스탯 증가 + 강화스킬+ | `enhanced_skill_plus` |
| 3 | 스탯 증가 + 지형적성 강화 | 적을 것이 없습니다 |
| 4 | 스탯 증가 + 특효 증가 | `effectiveness_bonus` |

3성은 수치가 아니라 연산입니다. **가장 높은 지형적성 하나가 한 단계 오릅니다.** 전무를
빼면 어느 학생이든 `s`가 정확히 하나라서 어느 것을 올릴지 갈리지 않고, 결과는 `ss`가
하나 생기는 것입니다.

4성의 `effectiveness_bonus`는 만분율이고, 그 학생 공격 타입의 특효에 더해집니다. 케이는
신비라 `mystic_effectiveness`가 오릅니다.

`enhanced_skill_plus`는 `skills`의 `Enhanced`와 같은 모양입니다. 전무 2성부터 강화스킬
수치가 통째로 이쪽으로 갈립니다.

### skills

키는 `Ex` `Basic` `Enhanced` `Sub` 넷입니다. 보스와 달리 배열의 축은 난이도가 아니라
**스킬 레벨**이고, 길이가 종류마다 다릅니다.

| 스킬 | 배열 길이 |
| --- | --- |
| `Ex` | 5 |
| `Basic` `Enhanced` `Sub` | 10 |

```json
"Sub": {
    "name": { "ko": "함께 걷는 친구", "ja": "", "en": "Those Who Walk Together" },
    "duration": 750,
    "buff": {
        "crit": {
            "amount": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            "scale": [13.1, 13.4, 14.1, 15.4, 16, 16.4, 19, 19.3, 19.6, 22.3]
        }
    }
}
```

`buff` 아래의 키는 올릴 스탯 이름이고 `lvl1_stats`의 필드 이름과 같습니다. `amount`는
고정값, `scale`은 비율입니다. 둘 중 쓰지 않는 쪽은 0으로 채웁니다.

스킬 레벨을 타지 않는 값(`duration`, `range`, `cost`, `frames`)은 배열이 아니라 그냥
숫자입니다.

강화스킬은 항상 수치 증가뿐이라 시뮬레이터에서 스킬로 두지 않고 학생 스탯에 미리
접어넣습니다. 그래도 파일에는 다른 스킬과 같은 자리에 적습니다.
