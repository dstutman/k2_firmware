use core::future::Future;

/// TODO: Debouncing
use crate::matrix::KeyState;
#[derive(Clone, Copy)]
pub enum Event {
    KeyUp { key: usize },
    KeyDown { key: usize },
}

pub trait Detector {
    fn step(&mut self, state: KeyState) -> Option<Event>;
}

pub enum SimpleKey<const ID: usize> {
    Waiting,
    Triggered,
}

impl<const ID: usize> Detector for SimpleKey<ID> {
    fn step(&mut self, state: KeyState) -> Option<Event> {
        match self {
            SimpleKey::Waiting => {
                if state == KeyState::Down {
                    *self = SimpleKey::Triggered;
                    Some(Event::KeyDown { key: ID })
                } else {
                    None
                }
            }
            SimpleKey::Triggered => {
                if state == KeyState::Up {
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
    pub fn step(&mut self, matrix_state: &[[KeyState; C]; R]) -> [Option<Event>; R * C] {
        let mut events = [None; R * C];
        for r in 0..R {
            for c in 0..C {
                events[r * C + c] = self.machines[r][c].step(matrix_state[r][c]);
            }
        }
        events
    }
}

pub trait EventSignaler {
    fn event(&mut self) -> &dyn Future<Output = ()>;
}

pub trait DeltaTimer {
    fn wait(&mut self, dt: usize) -> &dyn Future<Output = ()>;
}
