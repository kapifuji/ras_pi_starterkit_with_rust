// リレー制御して、LEDをON（1秒ごとにON/OFF繰り返し）

#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;

use stm32f4::stm32f446;
use stm32f4::stm32f446::interrupt;

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32f4::stm32f446::TIM2;
use stm32f4xx_hal as hal;

use hal::{
    gpio::{Output, PinState, PushPull, PB13},
    pac,
    prelude::*,
    timer::{CounterUs, Event},
};

static PB13: Mutex<RefCell<Option<PB13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static TIM2: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripheral = pac::Peripherals::take().unwrap();

    // クロック設定
    let rcc = peripheral.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .bypass_hse_oscillator()
        .sysclk(180.MHz())
        .pclk1(45.MHz()) // peripheral clock 1
        .pclk2(90.MHz()) // peripheral clock 2
        .freeze();

    // LED制御用 GPIO 設定
    let gpiob = peripheral.GPIOB.split();
    let pb13 = gpiob.pb13.into_push_pull_output_in_state(PinState::Low);

    cortex_m::interrupt::free(|cs| {
        PB13.borrow(cs).replace(Some(pb13));
    });

    // 割り込み登録
    unsafe {
        // TIM2割り込み有効化
        pac::NVIC::unmask(stm32f446::Interrupt::TIM2);
    }

    // 1sタイマ、更新割り込み有効
    let mut tim2 = peripheral.TIM2.counter_us(&clocks);
    tim2.start(1.secs()).unwrap();
    tim2.listen(Event::Update);

    cortex_m::interrupt::free(|cs| TIM2.borrow(cs).replace(Some(tim2)));

    loop {}
}

#[interrupt]
fn TIM2() {
    // 今回は更新割り込みのみを想定

    cortex_m::interrupt::free(|cs| {
        let mut tim2 = TIM2.borrow(cs).borrow_mut();
        let tim2 = tim2.as_mut();
        let tim2 = if let Some(tim2) = tim2 {
            tim2
        } else {
            panic!("TIM2 が見つかりませんでした。");
        };

        tim2.clear_interrupt(Event::all());

        let mut pb13 = PB13.borrow(cs).borrow_mut();
        let pb13 = pb13.as_mut();
        let pb13 = if let Some(pb13) = pb13 {
            pb13
        } else {
            panic!("PB1 が見つかりませんでした。");
        };

        pb13.toggle();
    })
}
