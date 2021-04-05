use tui::{
    style::{Color, Style},
    text::{Spans, Span},
    widgets::{ListItem},
};

// let messages = Vec::new::<String>();
pub struct Logger<'a>(Vec<ListItem<'a>>);

impl<'a> Logger<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn solution(&mut self, hash: &String, nounce: &String) {
        let hash = String::from(hash);
        let nounce = String::from(nounce);

        // Catergories message
        let mut line = vec![
            Span::raw(" [ "),
            Span::styled("OK", Style::default().fg(Color::Green)),
            Span::raw(" ]   "),
        ];

        // Add length
        let zn = crate::com::count_nounce(&hash);
        let length_str = format!("{:<5}", &zn);
        line.push(Span::raw(length_str));

        // Add sha256
        let zeros = String::from(&hash[..zn]);
        let zeros = Span::styled(zeros, Style::default().fg(Color::Cyan));
        let rest = String::from(&hash[zn..]);
        let rest  = Span::raw(rest);
        line.push(zeros);
        line.push(rest);
        line.push(Span::raw("   "));

        // Add nounce
        line.push(Span::raw(nounce));

        // Add to queue.
        let lines = vec![Spans::from(line)];
        let list_item = ListItem::new(lines).style(Style::default());
        self.0.push(list_item);
    }

    pub fn error(&mut self, message: &'a str) {
        // Catergories message
        let mut line = vec![
            Span::raw(" [ "),
            Span::styled("ERR", Style::default().fg(Color::Red)),
            Span::raw(" ]  "),
        ];
        
        line.push(Span::raw(message));

        // Add to queue.
        let lines = vec![Spans::from(line)];
        let list_item = ListItem::new(lines).style(Style::default());
        self.0.push(list_item);
    }


    pub fn info(&mut self, message: &'a str) {
        // Catergories message
        let mut line = vec![
            Span::raw(" [ "),
            Span::styled("INF", Style::default().fg(Color::LightBlue)),
            Span::raw(" ]  "),
        ];
        
        line.push(Span::raw(message));

        // Add to queue.
        let lines = vec![Spans::from(line)];
        let list_item = ListItem::new(lines).style(Style::default());
        self.0.push(list_item);
    }

    pub fn len(&mut self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self) -> ListItem<'a> {
        self.0.remove(0)
    }

    pub fn get(&self) -> &Vec<ListItem<'a>> {
        &self.0
    }
}
