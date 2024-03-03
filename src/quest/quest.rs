#[derive(Clone, PartialEq)]
pub struct Quest {
    pub quest_type: QuestType,
    pub spawn_info: crate::SpawnInfo,
    pub area_name: String,
    pub map_builder_args: crate::map_builder::MapBuilderArgs,
    pub reward: u32,
    pub turn_limit: u32,
    pub completed: bool,
    pub days_remaining: u8,
    pub started: bool,
}

impl Quest {
    pub fn get_name(&self) -> String {
        let mut name = "Hunt ".to_owned();
        name.push_str(&self.spawn_info.major_monsters.join(", "));
        name
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum QuestType {
    Hunt,
    Gather,
    Urgent,
}

impl QuestType {
    pub fn name(&self) -> String {
        match self {
            QuestType::Hunt => "Hunting Quest".to_string(),
            QuestType::Gather => "Gathering Quest".to_string(),
            QuestType::Urgent => "Urgent Quest".to_string(),
        }
    }

    pub fn short_name(&self) -> String {
        match self {
            QuestType::Hunt => "Hunt".to_string(),
            QuestType::Gather => "Gather".to_string(),
            QuestType::Urgent => "Urgent".to_string(),
        }
    }
}
