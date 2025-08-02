use log::{error, info};
pub struct GapBuffer {
    pub buffer: Vec<char>,
    pub capacity: usize,
    pub gap_start: usize,
    pub gap_end: usize,
}

impl GapBuffer {
    pub fn from_str(text: &str, capacity: usize) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let length = chars.len();
        let mut buffer: Vec<char> = Vec::with_capacity(capacity + length);
        buffer.extend_from_slice(&chars);
        buffer.resize(capacity + length, '\0');
        GapBuffer {
            buffer,
            capacity,
            gap_start: length,
            gap_end: length + capacity - 1,
        }
    }
    fn move_gap_left(&mut self, index: usize) {
        while self.gap_start > index {
            self.gap_start -= 1;
            self.gap_end -= 1;
            self.buffer[self.gap_end + 1] = self.buffer[self.gap_start];
            self.buffer[self.gap_start] = '\0';
        }
    }

    fn move_gap_right(&mut self, index: usize) {
        while self.gap_start < index {
            self.gap_start += 1;
            self.gap_end += 1;
            self.buffer[self.gap_start - 1] = self.buffer[self.gap_end];
            self.buffer[self.gap_end] = '\0';
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let new_size = new_capacity + self.buffer.len();
        let mut new_buffer = Vec::with_capacity(new_size);
        new_buffer.extend_from_slice(&self.buffer[..self.gap_start]);
        for _ in 0..new_capacity {
            new_buffer.push('\0');
        }
        let gap_end = new_buffer.len();
        new_buffer.extend_from_slice(&self.buffer[self.gap_end + 1..]);
        self.gap_end = gap_end;
        self.buffer = new_buffer;
    }

    pub fn insert_char(&mut self, c: char) {
        let gap_range = self.gap_end - self.gap_start;
        if gap_range == 1 {
            self.grow();
        }
        self.buffer[self.gap_start] = c;
        self.gap_start += 1;
    }

    pub fn delete_char(&mut self) {
        if self.gap_start == 0 {
            return;
        }
        self.gap_start -= 1;
        self.buffer[self.gap_start] = '\0';
    }

    pub fn move_gap(&mut self, index: usize) {
        let gap_size = self.gap_end - self.gap_start;
        if index + gap_size > self.buffer.len() {
            error!("Gap will overflow the buffer if moved to this index.");
            return;
        }
        if index == self.gap_start {
            info!("Gap is already positioned on this index.");
            return;
        }
        if index < self.gap_start {
            self.move_gap_left(index);
        }
        if index > self.gap_start {
            self.move_gap_right(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let gap_buffer = GapBuffer::from_str("Hello", 4);
        assert_eq!(gap_buffer.gap_start, 5);
        assert_eq!(gap_buffer.gap_end, 8);
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', 'e', 'l', 'l', 'o', '\0', '\0', '\0', '\0']
        );
    }

    #[test]
    fn test_move_gap_left() {
        let mut gap_buffer = GapBuffer::from_str("Hello", 3);
        gap_buffer.move_gap_left(1);
        assert_eq!(gap_buffer.gap_start, 1);
        assert_eq!(gap_buffer.gap_end, 3);
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', '\0', '\0', '\0', 'e', 'l', 'l', 'o']
        )
    }

    #[test]
    fn test_move_gap_right() {
        let mut gap_buffer = GapBuffer::from_str("Hello", 3);
        gap_buffer.move_gap_left(1);
        gap_buffer.move_gap_right(5);
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', 'e', 'l', 'l', 'o', '\0', '\0', '\0']
        );
    }

    #[test]
    fn test_move_gap() {
        let mut gap_buffer = GapBuffer::from_str("Hello", 3);
        gap_buffer.move_gap(2);
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', 'e', '\0', '\0', '\0', 'l', 'l', 'o']
        );
        // should do nothing
        gap_buffer.move_gap(7);
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', 'e', '\0', '\0', '\0', 'l', 'l', 'o']
        );
        gap_buffer.move_gap(3);
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', 'e', 'l', '\0', '\0', '\0', 'l', 'o']
        )
    }

    #[test]
    fn test_grow() {
        let mut gap_buffer = GapBuffer::from_str("Hello", 3);
        gap_buffer.move_gap(1);
        gap_buffer.grow();
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', '\0', '\0', '\0', '\0', '\0', '\0', 'e', 'l', 'l', 'o']
        )
    }

    #[test]
    fn test_insert_with_move() {
        let mut gap_buffer = GapBuffer::from_str("Hello", 2);
        gap_buffer.insert_char(' ');
        assert_eq!(
            &gap_buffer.buffer[..],
            &['H', 'e', 'l', 'l', 'o', ' ', '\0', '\0', '\0']
        );
    }
}
