use std::fmt::Display;
use database::model::library_entry::Model as LibraryEntry;

#[derive(Clone)]
pub struct Queue {
    queue: Vec<LibraryEntry>,
    current: i32
}

impl Queue {
    pub fn new() -> Self {
        Self {
            queue: vec![],
            current: -1
        }
    }

    pub fn add(&mut self, track: LibraryEntry) {
        self.queue.push(track);
    }

    pub fn next(&mut self) -> Option<LibraryEntry> {
        if self.queue.len() <= (self.current + 1) as usize {
            None
        } else {
            self.current += 1;
            self.queue.get(self.current as usize).map(|track| track.clone())
        }
    }

    pub fn prev(&mut self) -> Option<LibraryEntry> {
        if self.current < 1 {
            None
        } else {
            self.current -= 1;
            self.queue.get(self.current as usize).map(|track| track.clone())
        }
    }

    pub fn clear(&mut self) {
        self.current = -1;
        self.queue.clear();
    }
}

impl FromIterator<LibraryEntry> for Queue {
    fn from_iter<T: IntoIterator<Item=LibraryEntry>>(iter: T) -> Self {
        let mut queue = Queue::new();
        for track in iter {
            queue.add(track);
        }
        queue
    }
}

impl Iterator for Queue {
    type Item = LibraryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Queue([")?;
        for (i, track) in self.queue.iter().enumerate() {
            if i != 0 {
                write!(f, " ,")?;
            }
            write!(f, "{}", track.id)?;
        }
        write!(f, "])")?;
        Ok(())
    }
}
