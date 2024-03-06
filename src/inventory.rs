pub struct Inventory {
    pub money: u32,
    pub armor_level: u32,
    pub consumables: Vec<String>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            money: 0,
            armor_level: 0,
            consumables: vec![],
        }
    }
}
