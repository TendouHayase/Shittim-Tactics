#[derive(Debug, Clone, Copy)]
pub struct ExParams {
    pub coefficient: ExCoefficient,
    pub cost: u8,
    pub duration: u16,
    pub frames: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct ExCoefficient {
    pub first_damage_num: u16,
    pub second_damage_num: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct BasicCoefficient {
    pub first_damage: u16,
    pub second_damage: u16,
}

pub struct SubCoefficient {
    pub crit: u16,
}
