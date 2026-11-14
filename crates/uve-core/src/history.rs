#[derive(Debug, Clone, PartialEq)]
pub struct History<T> {
    past: Vec<T>,
    current: T,
    future: Vec<T>,
    limit: usize,
}

impl<T: Clone> History<T> {
    pub fn new(initial: T, limit: usize) -> Self {
        Self {
            past: Vec::new(),
            current: initial,
            future: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn current(&self) -> &T {
        &self.current
    }

    pub fn apply(&mut self, next: T) {
        self.past.push(self.current.clone());
        if self.past.len() > self.limit {
            self.past.remove(0);
        }
        self.current = next;
        self.future.clear();
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.past.pop() else {
            return false;
        };
        self.future.push(self.current.clone());
        self.current = previous;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };
        self.past.push(self.current.clone());
        self.current = next;
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undoes_and_redoes_edits() {
        let mut history = History::new(0, 10);
        history.apply(1);
        history.apply(2);
        assert_eq!(*history.current(), 2);
        assert!(history.undo());
        assert_eq!(*history.current(), 1);
        assert!(history.redo());
        assert_eq!(*history.current(), 2);
    }

    #[test]
    fn applying_after_undo_discards_redo_branch() {
        let mut history = History::new("a", 10);
        history.apply("b");
        assert!(history.undo());
        history.apply("c");
        assert!(!history.can_redo());
        assert_eq!(*history.current(), "c");
    }
}
