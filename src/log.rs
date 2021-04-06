
#[derive(Clone)]
pub enum LogMessage {
    Solution { hash: String, nounce: String },
    Info(String),
    Error(String),
}
pub struct Logger (Vec<LogMessage>);

impl Logger {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn solution(&mut self, hash: &String, nounce: &String) {
        let hash = String::from(hash);
        let nounce = String::from(nounce);

        self.0.push(LogMessage::Solution{
            hash,
            nounce,
        });
    }

    pub fn error(&mut self, message: &str) {
        self.0.push(LogMessage::Error(
            String::from(message)
        ));
    }


    pub fn info(&mut self, message: &str) {
        self.0.push(LogMessage::Info(
            String::from(message)
        ));
    }

    pub fn len(&mut self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self) -> LogMessage {
        self.0.remove(0)
    }

    pub fn get(&self) -> &Vec<LogMessage> {
        &self.0
    }
}
