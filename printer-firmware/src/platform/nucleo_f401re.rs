use stm32f4xx_hal::{
    prelude::*,
    serial::{self, Rx, Serial, Tx},
    stm32::{Peripherals, USART2},
};

pub(crate) struct Platform {
    pub sout: Tx<USART2>,
    pub sin: Rx<USART2>,
}

impl Platform {
    pub fn new() -> Self {
        // Get access to the device specific peripherals from the peripheral access crate
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

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

        let (tx, rx) = Serial::usart2(
            p.USART2,
            (tx, rx),
            serial::config::Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .map(|serial| serial.split())
        .unwrap_or_else(|_| unreachable!());

        Self { sin: rx, sout: tx }
    }
}
