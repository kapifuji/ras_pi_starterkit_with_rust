// PC8の出力をHにして、LEDを点灯させる。

#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32f4xx_hal as hal;

use hal::{gpio::PinState, pac, prelude::*};

#[entry]
fn main() -> ! {
    let peripheral = pac::Peripherals::take().unwrap();

    let rcc = peripheral.RCC.constrain();
    let _ = rcc
        .cfgr
        .use_hse(8.MHz())
        .bypass_hse_oscillator()
        .sysclk(180.MHz())
        .pclk1(45.MHz()) // peripheral clock 1
        .pclk2(90.MHz()) // peripheral clock 2
        .freeze();

    let gpioc = peripheral.GPIOC.split();
    let mut led = gpioc.pc8.into_push_pull_output_in_state(PinState::Low);
    led.set_high();

    loop {}
}
