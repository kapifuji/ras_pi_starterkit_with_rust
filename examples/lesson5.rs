// PB1, PB15, PB14, PB13 に接続されたLEDを順番に点灯させる。

#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;

use cortex_m_semihosting::hprintln;
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
    gpio::{Output, PinState, PushPull, PB1, PB13, PB14, PB15},
    pac,
    prelude::*,
    timer::{CounterUs, Event},
};

static LED1: Mutex<RefCell<Option<PB1<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static LED2: Mutex<RefCell<Option<PB15<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static LED3: Mutex<RefCell<Option<PB14<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static LED4: Mutex<RefCell<Option<PB13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

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
    let pb1 = gpiob.pb1.into_push_pull_output_in_state(PinState::High);
    let pb15 = gpiob.pb15.into_push_pull_output_in_state(PinState::High);
    let pb14 = gpiob.pb14.into_push_pull_output_in_state(PinState::High);
    let pb13 = gpiob.pb13.into_push_pull_output_in_state(PinState::High);

    cortex_m::interrupt::free(|cs| {
        LED1.borrow(cs).replace(Some(pb1));
        LED2.borrow(cs).replace(Some(pb15));
        LED3.borrow(cs).replace(Some(pb14));
        LED4.borrow(cs).replace(Some(pb13));
    });

    // 割り込み登録
    let core_peripheral = cortex_m::Peripherals::take().unwrap();
    let mut nvic = core_peripheral.NVIC;
    unsafe {
        // TIM2割り込み有効化
        cortex_m::peripheral::NVIC::unmask(stm32f446::Interrupt::TIM2);
        // 割り込み優先度を10に変更
        nvic.set_priority(stm32f446::Interrupt::TIM2, 10);
    }

    // 500msタイマ、更新割り込み有効
    let mut tim2 = peripheral.TIM2.counter_us(&clocks);
    tim2.start(500.millis()).unwrap();
    tim2.listen(Event::Update);

    cortex_m::interrupt::free(|cs| TIM2.borrow(cs).replace(Some(tim2)));

    loop {}
}

#[interrupt]
fn TIM2() {
    // 今回は更新割り込みのみを想定

    // 今回点灯するLED番号
    static LED_NUMBER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

    cortex_m::interrupt::free(|cs| {
        let mut led1 = LED1.borrow(cs).borrow_mut();
        let led1 = led1.as_mut();
        let led1 = if let Some(led) = led1 {
            led
        } else {
            panic!("LED1 が見つかりませんでした。");
        };

        let mut led2 = LED2.borrow(cs).borrow_mut();
        let led2 = led2.as_mut();
        let led2 = if let Some(led) = led2 {
            led
        } else {
            panic!("LED2 が見つかりませんでした。");
        };

        let mut led3 = LED3.borrow(cs).borrow_mut();
        let led3 = led3.as_mut();
        let led3 = if let Some(led) = led3 {
            led
        } else {
            panic!("LED3 が見つかりませんでした。");
        };

        let mut led4 = LED4.borrow(cs).borrow_mut();
        let led4 = led4.as_mut();
        let led4 = if let Some(led) = led4 {
            led
        } else {
            panic!("LED4 が見つかりませんでした。");
        };

        // 全消灯
        led1.set_high();
        led2.set_high();
        led3.set_high();
        led4.set_high();

        // 1つだけ点灯
        let mut number = LED_NUMBER.borrow(cs).borrow_mut();
        match *number {
            0 => led1.set_low(),
            1 => led2.set_low(),
            2 => led3.set_low(),
            3 => led4.set_low(),
            4 => led4.set_low(),
            5 => led3.set_low(),
            6 => led2.set_low(),
            7 => led1.set_low(),
            _ => {}
        }

        *number = (*number + 1) % 8; // LED番号更新

        let mut tim2 = TIM2.borrow(cs).borrow_mut();
        let tim2 = tim2.as_mut();
        let tim2 = if let Some(tim2) = tim2 {
            tim2
        } else {
            panic!("TIM2 が見つかりませんでした。");
        };

        tim2.clear_interrupt(Event::all());
    })
}
