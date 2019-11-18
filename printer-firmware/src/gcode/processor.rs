enum MoveType {
    Quick,
}

enum Command {
    LinearMove {
        /// X axis
        x: Option<f32>,
        /// Y axis
        y: Option<f32>,
        /// Z axis
        z: Option<f32>,
        /// Extruder axis
        e: Option<f32>,
    },
    /// Maximum movement speed
    FeedRate(f32),
}

pub struct Processor {}
impl Processor {
    pub fn process(&mut self) -> Iter {
        Iter { proc: self }
    }
}

pub struct Iter<'a> {
    proc: &'a mut Processor,
}
impl Iterator for Iter<'_> {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
