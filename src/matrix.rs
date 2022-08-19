use nrf52840_hal::{
    gpio::{Floating, Input, Output, Pin, PushPull},
    prelude::{InputPin, OutputPin},
};

const DEBOUNCE_LENGTH: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Up,
    Down,
}

/// Represents a matrix at the point in time `update` was last called.
pub struct Matrix<const R: usize, const C: usize> {
    pub state: [[KeyState; C]; R],
    counters: [[u8; C]; R],
    rows: [Pin<Output<PushPull>>; R],
    columns: [Pin<Input<Floating>>; C],
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    pub fn new(mut rows: [Pin<Output<PushPull>>; R], columns: [Pin<Input<Floating>>; C]) -> Self {
        for row in rows.as_mut_slice() {
            // unwrap: infallible
            row.set_low().unwrap();
        }

        Self {
            state: [[KeyState::Up; C]; R],
            counters: [[0; C]; R],
            rows,
            columns,
        }
    }

    // This is a hot loop for keyboard scanning in terms of power
    // consumption. Optimizing things like column-pulldown control
    // to a single cycle may have a meaningful impact on time-average
    // power consumption.
    pub fn update(mut self) -> Self {
        // Pull down the columns
        let columns = self.columns.map(|column| column.into_pulldown_input());

        for (row_index, row) in self.rows.iter_mut().enumerate() {
            // Push a row up so that we can scan the columns
            // unwrap: infallible
            row.set_high().unwrap();
            for (column_index, column) in columns.iter().enumerate() {
                // unwrap: infallible
                let measured_key_state = if column.is_high().unwrap() {
                    KeyState::Down
                } else {
                    KeyState::Up
                };
                let reported_key_state = self.state[row_index][column_index];

                // Increment counters for keys in a transition state.
                // Once they have been stable for `DEBOUNCE_LENGTH`
                // finalize the state change.
                if measured_key_state != reported_key_state {
                    self.counters[row_index][column_index] += 1;
                    // If the key has been stable for DEBOUNCE_LENGTH update
                    // its state and clear its counter.
                    if self.counters[row_index][column_index] == DEBOUNCE_LENGTH {
                        self.state[row_index][column_index] = measured_key_state;
                        self.counters[row_index][column_index] = 0;
                    }
                } else {
                    // If the key state matches the sample ensure the counter
                    // is reset so that all we only detect contiguously stable
                    // periods.
                    self.counters[row_index][column_index] = 0;
                }
            }
            // unwrap: infallible
            row.set_low().unwrap();
        }

        Self {
            columns: columns.map(|column| column.into_floating_input()),
            ..self
        }
    }
}
