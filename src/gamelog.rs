pub struct GameLog {
    pub entries: Vec<String>,
    pub dirty: bool,
}

impl GameLog {
    pub fn add(&mut self, text: &str) {
        self.entries.push(text.to_string());
        self.dirty = true;
    }
}
