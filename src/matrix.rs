// Low Power Matrix Supervisor (LPMS)

// TODO: Scheduling based on GPIOTE interrupts
// TODO: Improve standby power loss through pulldowns
// by scheduling high-impedance time

use hal::{
    gpio::{Disconnected, Level},
    prelude::OutputPin,
    prelude::*,
};
use nrf52840_hal::{
    self as hal,
    gpio::{Input, Output, Pin, PullDown, PushPull},
};

pub struct MatrixState<const R: usize, const C: usize> {
    state: [[bool; C]; R],
}

impl<const R: usize, const C: usize> MatrixState<R, C> {
    /// Constructs new (fully unmarked) matrix
    fn new() -> Self {
        Self {
            state: [[false; C]; R],
        }
    }

    /// Overwrites an entry
    fn set(&mut self, r: usize, c: usize, val: bool) {
        self.state[r][c] = val;
    }

    /// Checks if an entry is marked
    pub fn get(&mut self, r: usize, c: usize) -> bool {
        self.state[r][c]
    }

    pub fn any(&mut self) -> bool {
        self.state.flatten().contains(&true)
    }
}

enum SupervisorState<const R: usize, const C: usize> {
    Passive {
        rows: [Pin<Output<PushPull>>; R],
        columns: [Pin<Input<PullDown>>; C],
    },
    Dormant {
        rows: [Pin<Disconnected>; R],
        columns: [Pin<Disconnected>; C],
    },
    Active {
        rows: [Pin<Output<PushPull>>; R],
        columns: [Pin<Input<PullDown>>; C],
    },
}

pub struct MatrixSupervisor<const R: usize, const C: usize> {
    state: SupervisorState<R, C>,
    pub matrix: MatrixState<R, C>,
}

impl<const R: usize, const C: usize> MatrixSupervisor<R, C> {
    pub fn new(rows: [Pin<Disconnected>; R], columns: [Pin<Disconnected>; C]) -> Self {
        Self {
            state: SupervisorState::Active {
                rows: rows.map(|p| p.into_push_pull_output(Level::High)),
                columns: columns.map(|p| p.into_pulldown_input()),
            },
            matrix: MatrixState::new(),
        }
    }

    pub fn service(self) -> Self {
        match self.state {
            SupervisorState::Active { mut rows, columns } => {
                defmt::debug!("LPMS Mode: Active");
                let mut matrix: MatrixState<R, C> = MatrixState::new();

                for pin in rows.iter_mut() {
                    // unwrap: infallible
                    pin.set_low().unwrap();
                }

                for (row_index, row_pin) in rows.iter_mut().enumerate() {
                    // unwrap: infallible
                    row_pin.set_high().unwrap();
                    for (column_index, column_pin) in columns.iter().enumerate() {
                        // unwrap: infallible
                        matrix.set(row_index, column_index, column_pin.is_high().unwrap());
                    }
                    // unwrap: infallible
                    row_pin.set_low().unwrap();
                }

                // Key releases cannot be detected in the passive state, so if
                // any key is depressed we need to keep going through dormant and active
                // states. Conversely, if all keys are released we can go passive.
                if matrix.any() {
                    Self {
                        state: SupervisorState::Dormant {
                            rows: rows.map(|p| p.into_disconnected()),
                            columns: columns.map(|p| p.into_disconnected()),
                        },
                        matrix,
                    }
                } else {
                    // Set all columns high and go passive
                    for pin in &mut rows {
                        // unwrap: infallible
                        pin.set_high().unwrap();
                    }
                    Self {
                        state: SupervisorState::Passive { rows, columns },
                        matrix,
                    }
                }
            }
            // We have been woken from a passive state. Go active and service immediately.
            SupervisorState::Passive { rows, columns } => {
                defmt::debug!("LPMS Mode: Passive");
                Self {
                    state: SupervisorState::Active { rows, columns },
                    ..self
                }
                .service()
            }
            // We have been dormant for a cycle, so now we go active.
            SupervisorState::Dormant { rows, columns } => {
                defmt::debug!("LPMS Mode: Dormant");
                Self {
                    state: SupervisorState::Active {
                        rows: rows.map(|p| p.into_push_pull_output(Level::High)),
                        columns: columns.map(|p| p.into_pulldown_input()),
                    },
                    ..self
                }
                .service()
            }
        }
    }
}
