pub struct LogEntry {
    pub text: String,
    pub count: u32,
}

pub struct GameLog {
    pub entries: Vec<LogEntry>,
    pub pending: Option<String>,
    pub dirty: bool,
}

impl GameLog {
    pub fn add<S>(&mut self, text: S)
    where
        S: ToString,
    {
        let txt = text.to_string();
        let prev = self.entries.last_mut();

        if let Some(prev_entry) = prev {
            if txt == prev_entry.text {
                prev_entry.count += 1;
            } else {
                self.new_entry(txt);
            }
        } else {
            self.new_entry(txt);
        }

        self.dirty = true;
    }

    pub fn late_add(&mut self, text: &str) {
        self.pending = Some(text.to_string());
    }

    pub fn transfer_pending(&mut self) {
        if let Some(text) = self.pending.clone() {
            self.add(&text);
            self.pending = None;
        }
    }

    fn new_entry(&mut self, text: String) {
        self.entries.push(LogEntry { text, count: 1 });
    }
}
