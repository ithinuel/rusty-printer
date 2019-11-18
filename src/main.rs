#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
extern crate stm32f4xx_hal;
#[macro_use]
extern crate nb;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    prelude::*,
    stm32,
    timer::Timer,
};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32::Peripherals::take().unwrap();
 
    // Take ownership over the raw flash and rcc devices and convert them
    // into the corresponding HAL structs
    let rcc = dp.RCC.constrain();
 
    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze();
 
    // Acquire the GPIOC peripheral
    let gpioa = dp.GPIOA.split();
 
    // Configure gpio C pin 13 as a push-pull output. The `crh` register is
    // passed to the function in order to configure the port. For pins 0-7,
    // crl should be passed instead.
    let mut led = gpioa.pa5.into_push_pull_output();
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);
 
    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
    } 
}
