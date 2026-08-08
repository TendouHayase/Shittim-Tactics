use std::collections::HashSet;

use error::Error;
use serde::{Deserialize, Serialize};

use crate::{stat::StatKind, types::StatValueKind, utils::Ratio};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct GearTable {
    pub max_level_with_tier: [u8; Self::MAX_TIER],
    pub slots: GearSlotTable,
}

impl GearTable {
    pub const MAX_TIER: usize = 10;

    /// 애장품 제외
    pub const TOTAL_GEARS_COUNT: usize = 9;

    pub fn gear_stats(&self, kind: GearKind) -> &[GearStat] {
        match kind {
            GearKind::Hat => &self.slots.hat,
            GearKind::Gloves => &self.slots.gloves,
            GearKind::Shoes => &self.slots.shoes,
            GearKind::Bag => &self.slots.bag,
            GearKind::Badge => &self.slots.badge,
            GearKind::Hairpin => &self.slots.hairpin,
            GearKind::Amulet => &self.slots.amulet,
            GearKind::Wristwatch => &self.slots.wristwatch,
            GearKind::Necklace => &self.slots.necklace,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Error> {
        let gear_table: GearTable = parsing_json::read_json(path)?;

        gear_table.validate()?;

        Ok(gear_table)
    }

    fn validate(&self) -> Result<(), Error> {
        for i in 0..self.max_level_with_tier.len() - 1 {
            if self.max_level_with_tier[i] >= self.max_level_with_tier[i + 1] {
                return Err(Error::InvalidData(format!(
                    "The maximum level of the {i} tier of equipment must be less than the maximum level of the {} tier of equipment.",
                    i + 1
                )));
            }
        }

        for kind in GearKind::ALL {
            let mut set: HashSet<(StatKind, StatValueKind)> = HashSet::new();
            for stats in self.gear_stats(kind) {
                set.insert((stats.stat, stats.kind))
                    .then_some(())
                    .ok_or_else(|| {
                        Error::InvalidData(
                        "There are elements of the same type and value form in the same equipment"
                            .to_string(),
                    )
                    })?;

                for i in 0..stats.curve.len() - 1 {
                    if stats.curve[i][1] > stats.curve[i + 1][0] {
                        let stat = stats.stat;
                        return Err(Error::InvalidData(format!(
                            "The maximum {stat:?} of {num} tier {gear:?} must be smaller than the minimum {stat:?} of {} tier {gear:?}",
                            i + 1,
                            num = i,
                            gear = kind,
                        )));
                    }
                }

                for i in 0..stats.curve.len() {
                    if stats.curve[i][0] > stats.curve[i][1] {
                        let stat = stats.stat;
                        return Err(Error::InvalidData(format!(
                            "The minimum {stat:?} of {num} tier {gear:?} must be smaller than the maximum {stat:?} of {num} tier {gear:?}",
                            num = i,
                            gear = kind,
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

/// 애장품은 다른 장비들과 성질이 크게 달라 따로 분리
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum GearKind {
    Hat,
    Gloves,
    Shoes,
    Bag,
    Badge,
    Hairpin,
    Amulet,
    Wristwatch,
    Necklace,
}

impl GearKind {
    /// 전체 장비 목록
    pub const ALL: [GearKind; GearTable::TOTAL_GEARS_COUNT] = [
        GearKind::Hat,
        GearKind::Gloves,
        GearKind::Shoes,
        GearKind::Bag,
        GearKind::Badge,
        GearKind::Hairpin,
        GearKind::Amulet,
        GearKind::Wristwatch,
        GearKind::Necklace,
    ];
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearStat {
    pub stat: StatKind,
    pub kind: StatValueKind,
    pub curve: [[Ratio; 2]; GearTable::MAX_TIER],
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct GearSlotTable {
    pub hat: Vec<GearStat>,
    pub gloves: Vec<GearStat>,
    pub shoes: Vec<GearStat>,
    pub bag: Vec<GearStat>,
    pub badge: Vec<GearStat>,
    pub hairpin: Vec<GearStat>,
    pub amulet: Vec<GearStat>,
    pub wristwatch: Vec<GearStat>,
    pub necklace: Vec<GearStat>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// 워크스페이스 루트 기준이 아니라 크레이트 루트 기준으로 도는 것에 주의.
    const GEARS: &str = "../../data/tables/gears.json";

    fn table() -> GearTable {
        GearTable::from_file(GEARS).expect("failed to load gears")
    }

    fn ratio(num: i64, exp: u8) -> Ratio {
        Ratio::new(num, exp)
    }

    #[test]
    fn load_gears() {
        let table = table();

        assert_eq!(
            table.max_level_with_tier,
            [10, 20, 30, 40, 45, 50, 55, 60, 65, 70]
        );

        let hat_atk = &table.gear_stats(GearKind::Hat)[0];
        assert_eq!(hat_atk.stat, StatKind::Atk);
        assert_eq!(hat_atk.kind, StatValueKind::Scale);
        assert_eq!(hat_atk.curve[0], [ratio(5, 0), ratio(8, 0)]);
        assert_eq!(hat_atk.curve[9], [ratio(48, 0), ratio(50, 0)]);

        // 1~3티어가 [0, 0] 패딩인 행. 값 없음과 0을 같은 것으로 두기로 한 결과임.
        let necklace_atk = &table.gear_stats(GearKind::Necklace)[2];
        assert_eq!(necklace_atk.curve[0], [ratio(0, 0), ratio(0, 0)]);
        assert_eq!(necklace_atk.curve[9], [ratio(15, 0), ratio(18, 0)]);
    }

    /// `gear_stats`의 9줄 `match`가 한 칸 밀려도 컴파일은 통과함. 슬롯마다 다른 데이터로 짚음.
    #[test]
    fn gear_stats_maps_every_slot() {
        let table = table();

        // 앞 세 슬롯은 첫 행이 모두 atk/scale이라 행 개수와 1티어 값을 같이 봐야 갈림.
        assert_eq!(table.gear_stats(GearKind::Hat).len(), 2);
        assert_eq!(
            table.gear_stats(GearKind::Hat)[0].curve[0],
            [ratio(5, 0), ratio(8, 0)]
        );

        assert_eq!(table.gear_stats(GearKind::Gloves).len(), 3);
        assert_eq!(
            table.gear_stats(GearKind::Gloves)[0].curve[0],
            [ratio(4, 0), ratio(64, 1)]
        );

        assert_eq!(table.gear_stats(GearKind::Shoes).len(), 2);
        assert_eq!(
            table.gear_stats(GearKind::Shoes)[0].curve[0],
            [ratio(25, 1), ratio(4, 0)]
        );

        assert_eq!(table.gear_stats(GearKind::Bag).len(), 3);
        assert_eq!(table.gear_stats(GearKind::Bag)[0].stat, StatKind::Hp);
        assert_eq!(
            table.gear_stats(GearKind::Bag)[0].curve[0],
            [ratio(375, 0), ratio(600, 0)]
        );

        assert_eq!(table.gear_stats(GearKind::Badge).len(), 4);
        assert_eq!(
            table.gear_stats(GearKind::Badge)[1].stat,
            StatKind::RecoveryBoost
        );

        assert_eq!(table.gear_stats(GearKind::Hairpin).len(), 3);
        assert_eq!(table.gear_stats(GearKind::Hairpin)[1].stat, StatKind::CcRes);

        assert_eq!(
            table.gear_stats(GearKind::Amulet)[0].stat,
            StatKind::CritRes
        );
        assert_eq!(
            table.gear_stats(GearKind::Wristwatch)[0].stat,
            StatKind::Crit
        );
        assert_eq!(
            table.gear_stats(GearKind::Necklace)[0].stat,
            StatKind::Healing
        );
    }

    /// `[GearKind; 9]`는 길이만 컴파일 타임에 보장됨. 한 variant가 두 번 들어가면 다른 하나가
    /// 빠지고, 그 슬롯은 `validate`가 영영 보지 않게 됨.
    #[test]
    fn gear_kind_all_covers_every_variant() {
        let unique: HashSet<GearKind> = GearKind::ALL.into_iter().collect();

        assert_eq!(unique.len(), GearTable::TOTAL_GEARS_COUNT);
    }

    /// 같은 `bag`에 hp/amount와 hp/scale이 정상적으로 공존하므로 중복 판정은 `stat`만으로는 안 됨.
    #[test]
    fn stat_and_kind_pairs_are_unique_in_a_slot() {
        let table = table();

        for kind in GearKind::ALL {
            let stats = table.gear_stats(kind);
            let unique: HashSet<(StatKind, StatValueKind)> =
                stats.iter().map(|s| (s.stat, s.kind)).collect();

            assert_eq!(unique.len(), stats.len(), "duplicated stat in {kind:?}");
        }
    }

    #[test]
    fn validate_rejects_flat_max_level() {
        let mut table = table();
        table.max_level_with_tier[3] = table.max_level_with_tier[2];

        assert!(table.validate().is_err());
    }

    #[test]
    fn validate_rejects_tier_regression() {
        let mut table = table();
        // 4티어 만렙 값을 5티어 1렙 값보다 크게.
        table.slots.hat[0].curve[3][1] = ratio(9999, 0);

        assert!(table.validate().is_err());
    }

    /// 마지막 티어를 고르는 것이 핵심임. 쌍 내부 검사가 `len() - 1`까지만 돌면 여기가 뚫림.
    #[test]
    fn validate_rejects_reversed_last_tier() {
        let mut table = table();
        table.slots.hat[0].curve[9].swap(0, 1);

        assert!(table.validate().is_err());
    }

    /// 소수가 f64를 거쳐도 자릿수 그대로 복원되는지. `GrowthDelta`와 달리 배열 두 겹을 거침.
    #[test]
    fn curve_keeps_decimals() {
        let table = table();

        let gloves_atk = &table.gear_stats(GearKind::Gloves)[0];
        assert_eq!(gloves_atk.curve[0][1].num(), 64);
        assert_eq!(gloves_atk.curve[0][1].den(), 10);
        assert_eq!(gloves_atk.curve[9][1].den(), 1);

        // 2.5 < 4. exp가 큰 쪽이 값도 크다고 보던 옛 `Ratio::cmp`라면 여기서 뒤집힘.
        let shoes_atk = &table.gear_stats(GearKind::Shoes)[0];
        assert!(shoes_atk.curve[0][0] < shoes_atk.curve[0][1]);
    }
}
