/// TODO: Debouncing
use crate::matrix::MatrixState;
#[derive(Clone, Copy)]
pub enum Event {
    KeyUp { key: usize },
    KeyDown { key: usize },
}

pub trait Detector {
    fn step(&mut self, key_state: bool) -> Option<Event>;
}

pub enum SimpleKey<const ID: usize> {
    Waiting,
    Triggered,
}

impl<const ID: usize> Detector for SimpleKey<ID> {
    fn step(&mut self, key_state: bool) -> Option<Event> {
        match self {
            SimpleKey::Waiting => {
                if key_state {
                    *self = SimpleKey::Triggered;
                    Some(Event::KeyDown { key: ID })
                } else {
                    None
                }
            }
            SimpleKey::Triggered => {
                if !key_state {
                    *self = SimpleKey::Waiting;
                    Some(Event::KeyUp { key: ID })
                } else {
                    None
                }
            }
        }
    }
}

pub struct Mapper<'a, const R: usize, const C: usize> {
    machines: [[&'a mut dyn Detector; C]; R],
}

impl<'a, const R: usize, const C: usize> Mapper<'a, R, C> {
    pub fn new(machines: [[&'a mut dyn Detector; C]; R]) -> Self {
        Self { machines }
    }

    /// Only one event per key may occur during a step.
    pub fn step(&mut self, matrix_state: &MatrixState<R, C>) -> [Option<Event>; R * C] {
        let mut events = [None; R * C];
        for r in 0..R {
            for c in 0..C {
                events[r * C + c] = self.machines[r][c].step(matrix_state.get(r, c));
            }
        }
        events
    }
}
