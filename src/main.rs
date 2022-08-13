#![no_std]
#![no_main]
#![feature(slice_flatten)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use hal::{pac, prelude::OutputPin};
use nrf52840_hal as hal;

mod matrix;
use matrix::MatrixSupervisor;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::debug!("Entered: main()");

    // unwrap: Must acquire peripherals on startup.
    let peripherals = pac::Peripherals::take().unwrap();

    // TODO: Enable GPIO on NFC pins by writing UICR

    let gpio0 = hal::gpio::p0::Parts::new(peripherals.P0);
    let gpio1 = hal::gpio::p1::Parts::new(peripherals.P1);

    // How the Low Power Matrix Supervisor (LPMS) works:
    // By default all rows are driven to logic high, and
    // all columns are pulled to logic low. This is the "passive" state.
    // The supervisor waits for a column pin state change.
    // It then starts to "actively" scan until all keys are released.
    // During a scan it drives the columns and reads the rows. It then
    // waits in high-impedance 'dormant" state for a configurable period
    // after which it starts reading again.
    // Once all keys are released it re-enters the passive state.

    let mut supervisor: MatrixSupervisor<6, 5> = MatrixSupervisor::new(
        [
            gpio0.p0_26.degrade(),
            gpio0.p0_31.degrade(),
            gpio0.p0_29.degrade(),
            gpio0.p0_02.degrade(),
            gpio1.p1_15.degrade(),
            gpio1.p1_13.degrade(),
        ],
        [
            gpio0.p0_10.degrade(),
            gpio0.p0_09.degrade(),
            gpio0.p0_24.degrade(),
            gpio0.p0_22.degrade(),
            gpio0.p0_20.degrade(),
        ],
    );

    let mut blue_led = gpio0
        .p0_04
        .into_push_pull_output(hal::gpio::Level::Low)
        .degrade();

    loop {
        supervisor = supervisor.service();
        if supervisor.matrix.any() {
            // unwrap: infallible
            blue_led.set_high().unwrap();
        } else {
            // unwrap: infallible
            blue_led.set_low().unwrap();
        }
        for j in 0..5 {
            for i in 0..6 {
                if supervisor.matrix.get(i, j) {
                    defmt::info!("Key depressed ({}, {})", i, j);
                }
            }
        }
    }
}
