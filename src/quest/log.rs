use super::quest::{Quest, QuestType};

pub struct QuestLog {
    pub entries: Vec<Quest>,
}

impl QuestLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_quest(&mut self, rng: &mut rltk::RandomNumberGenerator, difficulty: i32) {
        let area_info = crate::data::get_random_area(rng);
        let spawn_info = crate::spawn::info::generate_spawn_info(rng, difficulty);
        let quest_difficulty = spawn_info.difficulty;
        let name_copy = area_info.name.clone();

        let quest = Quest {
            quest_type: QuestType::Hunt,
            spawn_info,
            area_name: area_info.name,
            map_builder_args: crate::map_builder::MapBuilderArgs {
                builder_type: area_info.map_type,
                height: 40 + 10 * quest_difficulty,
                width: 40 + 10 * quest_difficulty,
                name: name_copy,
                map_color: area_info.color,
            },
            reward: 120,
            turn_limit: 300,
            completed: false,
            days_remaining: 3,
            started: false,
        };

        self.entries.push(quest);
    }

    pub fn advance_day(&mut self) {
        // remove all quests that have no days remaining
        self.entries.retain(|quest| quest.days_remaining > 1);

        // update the days on the remaining quests
        for quest in self.entries.iter_mut() {
            quest.days_remaining -= 1;
        }
    }
}
