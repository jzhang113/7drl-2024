pub struct Inventory {
    pub money: u32,
    pub weapon: Box<dyn crate::weapon::Weapon>,
    pub armor_level: u32,
    pub consumables: Vec<String>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            money: 0,
            weapon: Box::new(crate::weapon::lance::Lance::new()),
            armor_level: 0,
            consumables: vec![],
        }
    }
}
