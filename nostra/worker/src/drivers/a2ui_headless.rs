use anyhow::Result;

pub struct A2UiHeadlessDriver {
    pub virtual_dom: String,
}

impl Default for A2UiHeadlessDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl A2UiHeadlessDriver {
    pub fn new() -> Self {
        Self {
            virtual_dom: String::new(),
        }
    }

    pub fn process_message(&mut self, msg: &str) -> Result<()> {
        // Mock implementation: Just append to virtual DOM log
        self.virtual_dom.push_str(msg);
        self.virtual_dom.push('\n');
        Ok(())
    }
}
