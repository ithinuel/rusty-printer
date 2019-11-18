pub struct Queue;

pub enum Error<T> {
    QueueIsFull(T),
}

impl Queue {
    pub fn push(&mut self, line: Vec<GCode>) -> Result<(), Error<Vec<GCode>>> {
        unimplemented!()
    }

    pub fn pop(&mut self, from: &State) -> Option<PartialState> {
        unimplemented!()
    }
    pub fn is_full(&self) -> bool {
        false
    }
}
