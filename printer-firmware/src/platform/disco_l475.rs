use stm32l4xx_hal::{
    prelude::*,
    serial::{self, Rx, Serial, Tx},
    stm32::{Peripherals, USART1},
};

pub(crate) struct Platform {
    pub sout: Tx<USART1>,
    pub sin: Rx<USART1>,
    pub name: &'static str,
}

impl Platform {
    pub fn take() -> Self {
        // Get access to the device specific peripherals from the peripheral access crate
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        // Take ownership over the raw flash and rcc devices and convert them
        // into the corresponding HAL structs
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);

        // Freeze the configuration of all the clocks in the system and store
        // the frozen frequencies in `clocks`
        let clocks = rcc.cfgr.sysclk(80.mhz()).freeze(&mut flash.acr, &mut pwr);

        // Acquire the GPIOC peripheral
        let mut gpioa = p.GPIOB.split(&mut rcc.ahb2);

        let tx = gpioa.pb6.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
        let rx = gpioa.pb7.into_af7(&mut gpioa.moder, &mut gpioa.afrl);

        let (tx, rx) = Serial::usart1(
            p.USART1,
            (tx, rx),
            serial::Config::default().baudrate(115_200.bps()),
            clocks,
            &mut rcc.apb2,
        )
        .split();

        Self {
            sin: rx,
            sout: tx,
            name: "disco-l475-iot01a",
        }
    }
}
