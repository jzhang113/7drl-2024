use specs::Entity;

pub struct MissionInfo {
    pub remaining: Vec<Entity>,
}

impl MissionInfo {
    pub fn new() -> Self {
        Self {
            remaining: Vec::new(),
        }
    }

    pub fn add(&mut self, entity: Entity) {
        self.remaining.push(entity);
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(index) = self.remaining.iter().position(|value| *value == entity) {
            self.remaining.swap_remove(index);
        }
    }

    pub fn is_done(&self) -> bool {
        self.remaining.is_empty()
    }

    pub fn reset(&mut self) {
        self.remaining = Vec::new();
    }
}
