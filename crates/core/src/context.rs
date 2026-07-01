use crate::{
    Position,
    base::BaseStats,
    skill::{Effect, EffectKind, SkillType},
};

pub struct CasterContext<'a> {
    pub stats: &'a mut BaseStats,
    pub effects: &'a mut Vec<Effect>,
    pub position: &'a mut Position,
    pub skill_type: SkillType,
}

pub struct TargetContext<'a> {
    pub stats: &'a mut BaseStats,
    pub effects: &'a mut Vec<Effect>,
    pub position: &'a mut Position,
}

pub struct SkillContext<'a> {
    pub name: &'a str,
    pub caster: CasterContext<'a>,
    pub targets: Vec<TargetContext<'a>>,
}

impl<'a> CasterContext<'a> {
    pub fn atk(&self) -> u32 {
        let result = self.stats.atk;
        let mut scale = 100;
        let mut inc = 0;
        for effect in self.effects.iter() {
            if let EffectKind::Buff {
                ty: t,
                duration: d,
                scale: s,
                amount: i,
            } = &effect.kind
            {
                match t {
                    crate::skill::Buff::Atk => {
                        scale *= s;
                        inc += i;
                    }
                    _ => (),
                }
            }
        }

        (result + inc) * (scale as u32)
    }

    pub fn hp(&self) -> u64 {
        self.stats.hp
    }

    pub fn decrease_hp(&mut self, amount: u64) {
        self.stats.hp = self.stats.hp.saturating_sub(amount);
    }
}

impl<'a> TargetContext<'a> {
    pub fn atk(&self) -> u32 {
        let result = self.stats.atk;
        let mut scale = 100;
        let mut inc = 0;
        for effect in self.effects.iter() {
            if let EffectKind::Buff {
                ty: t,
                duration: d,
                scale: s,
                amount: i,
            } = &effect.kind
            {
                match t {
                    crate::skill::Buff::Atk => {
                        scale *= s;
                        inc += i;
                    }
                    _ => (),
                }
            }
        }

        (result + inc) * (scale as u32)
    }

    pub fn hp(&self) -> u64 {
        self.stats.hp
    }

    pub fn decrease_hp(&mut self, amount: u64) {}
}
