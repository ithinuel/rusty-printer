#![no_std]
#![no_main]

extern crate nb;
extern crate panic_halt;

use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use stm32f4xx_hal::{
    prelude::*,
    serial::{self, Serial},
    stm32,
};

#[derive(Debug)]
enum Error<E> {
    Io(E),
    Parser(gcode::Error),
}
impl<E> From<gcode::Error> for Error<E> {
    fn from(f: gcode::Error) -> Error<E> {
        Error::Parser(f)
    }
}

struct SerialIterator<B: Read<u8>>(B);
impl<B: Read<u8>> Iterator for SerialIterator<B> {
    type Item = Result<char, Error<B::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.read() {
            Ok(byte) => Some(Ok(byte.into())),
            Err(nb::Error::WouldBlock) => None,
            Err(nb::Error::Other(o)) => Some(Err(Error::Io(o))),
        }
    }
}

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let p = stm32::Peripherals::take().unwrap_or_else(|| unreachable!());

    // Take ownership over the raw flash and rcc devices and convert them
    // into the corresponding HAL structs
    let rcc = p.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Acquire the GPIOC peripheral
    let gpioa = p.GPIOA.split();

    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7();

    let (mut tx, rx) = Serial::usart2(
        p.USART2,
        (tx, rx),
        serial::config::Config::default().baudrate(115_200.bps()),
        clocks,
    )
    .map(|serial| serial.split())
    .unwrap_or_else(|_| unreachable!());

    let it = SerialIterator(rx);
    // writeln!(tx, "size_of<SerialIterator>: {}", core::mem::size_of_val(&it)).unwrap();

    let mut rx = gcode::Parser::new(it);
    // use core::fmt::Write;
    // writeln!(tx, "size_of<Parser<SerialIterator>>: {}", core::mem::size_of_val(&rx)).unwrap();

    loop {
        match rx.next() {
            Some(Ok(b)) => {
                // writeln!(tx, "{:?}", b).unwrap();
            }
            Some(Err(e)) => {
                // writeln!(tx, "err: {:?}", e).unwrap();
            }
            _ => {}
        }
    }
}
