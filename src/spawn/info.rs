#[derive(PartialEq, Clone, Default)]
pub struct SpawnInfo {
    pub major_monsters: Vec<String>,
    pub minor_monsters: Vec<String>,
    pub resources: Vec<String>,
    pub difficulty: i32,
}

pub fn generate_spawn_info(
    rng: &mut rltk::RandomNumberGenerator,
    target_difficulty: i32,
) -> SpawnInfo {
    let mut curr_difficulty = 0;
    let mut major_monsters = Vec::new();

    while curr_difficulty < target_difficulty {
        let rand_index = rng.range(0, super::spawner::MONSTERS.len());
        let (name, (difficulty, _)) = super::spawner::MONSTERS.iter().nth(rand_index).unwrap();

        curr_difficulty += difficulty;
        major_monsters.push(name.clone());

        // chance to early quit
        if rng.rand::<f32>() < curr_difficulty as f32 / target_difficulty as f32 {
            break;
        }
    }

    SpawnInfo {
        major_monsters,
        minor_monsters: vec![],
        resources: vec![],
        difficulty: curr_difficulty,
    }
}
