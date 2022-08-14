#![no_std]
#![no_main]
#![feature(slice_flatten)]
// Hopefully just using multiplication wont cause major issues.
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use hal::{pac, prelude::OutputPin};
use nrf52840_hal as hal;

mod matrix;
use matrix::MatrixSupervisor;

mod event_mapper;
use event_mapper::{Event, Mapper, SimpleKey};

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

    let mut k00 = SimpleKey::<0>::Waiting;
    let mut k01 = SimpleKey::<1>::Waiting;
    let mut k02 = SimpleKey::<2>::Waiting;
    let mut k03 = SimpleKey::<3>::Waiting;
    let mut k04 = SimpleKey::<4>::Waiting;
    let mut k10 = SimpleKey::<5>::Waiting;
    let mut k11 = SimpleKey::<6>::Waiting;
    let mut k12 = SimpleKey::<7>::Waiting;
    let mut k13 = SimpleKey::<8>::Waiting;
    let mut k14 = SimpleKey::<9>::Waiting;
    let mut k20 = SimpleKey::<10>::Waiting;
    let mut k21 = SimpleKey::<11>::Waiting;
    let mut k22 = SimpleKey::<12>::Waiting;
    let mut k23 = SimpleKey::<13>::Waiting;
    let mut k24 = SimpleKey::<14>::Waiting;
    let mut k30 = SimpleKey::<15>::Waiting;
    let mut k31 = SimpleKey::<16>::Waiting;
    let mut k32 = SimpleKey::<17>::Waiting;
    let mut k33 = SimpleKey::<18>::Waiting;
    let mut k34 = SimpleKey::<19>::Waiting;
    let mut k40 = SimpleKey::<20>::Waiting;
    let mut k41 = SimpleKey::<21>::Waiting;
    let mut k42 = SimpleKey::<22>::Waiting;
    let mut k43 = SimpleKey::<23>::Waiting;
    let mut k44 = SimpleKey::<24>::Waiting;
    let mut k50 = SimpleKey::<25>::Waiting;
    let mut k51 = SimpleKey::<26>::Waiting;
    let mut k52 = SimpleKey::<27>::Waiting;
    let mut k53 = SimpleKey::<28>::Waiting;
    let mut k54 = SimpleKey::<29>::Waiting;

    let mut mapper = Mapper::new([
        [&mut k00, &mut k01, &mut k02, &mut k03, &mut k04],
        [&mut k10, &mut k11, &mut k12, &mut k13, &mut k14],
        [&mut k20, &mut k21, &mut k22, &mut k23, &mut k24],
        [&mut k30, &mut k31, &mut k32, &mut k33, &mut k34],
        [&mut k40, &mut k41, &mut k42, &mut k43, &mut k44],
        [&mut k50, &mut k51, &mut k52, &mut k53, &mut k54],
    ]);

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

        let events = mapper.step(&supervisor.matrix);

        for maybe_event in events {
            if let Some(event) = maybe_event {
                match event {
                    Event::KeyDown { key } => {
                        defmt::info!("Key down: {}", key);
                    }
                    Event::KeyUp { key } => {
                        defmt::info!("Key up: {}", key);
                    }
                };
            }
        }
    }
}
