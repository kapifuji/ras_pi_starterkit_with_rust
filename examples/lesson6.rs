// PA1 に接続されたLEDをゆっくりと点滅させる。

// TIMの割り込みごとにPWMのDUTY書き換え

#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;

use stm32f4::stm32f446::interrupt;
use stm32f4::stm32f446::{self};

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32f4::stm32f446::{TIM2, TIM3};
use stm32f4xx_hal as hal;

use hal::{pac, prelude::*, timer::CounterUs, timer::Event, timer::PwmChannel, timer::C2};

static TIM2_PWM_CH2: Mutex<RefCell<Option<PwmChannel<TIM2, C2>>>> = Mutex::new(RefCell::new(None));
static TIM3: Mutex<RefCell<Option<CounterUs<TIM3>>>> = Mutex::new(RefCell::new(None));

static DUTY_TABLE: [u16; 100] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    49, 48, 47, 46, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26,
    25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
];

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
    let gpioa = peripheral.GPIOA.split();
    let pa1 = gpioa.pa1.into_alternate(); // TIM2-ch2

    // 割り込み登録
    let core_peripheral = cortex_m::Peripherals::take().unwrap();
    let mut nvic = core_peripheral.NVIC;
    unsafe {
        // TIM3割り込み有効化
        cortex_m::peripheral::NVIC::unmask(stm32f446::Interrupt::TIM3);
        // 割り込み優先度を10に変更
        nvic.set_priority(stm32f446::Interrupt::TIM3, 10);
    }

    // 20kHz PWM
    let mut tim2_pwm_ch2 = peripheral.TIM2.pwm_hz(pa1, 20.kHz(), &clocks).split();
    tim2_pwm_ch2.set_duty(0);
    tim2_pwm_ch2.enable();
    cortex_m::interrupt::free(|cs| TIM2_PWM_CH2.borrow(cs).replace(Some(tim2_pwm_ch2)));

    // 10msタイマ、更新割り込み有効
    let mut tim3 = peripheral.TIM3.counter_us(&clocks);
    tim3.start(10.millis()).unwrap();
    tim3.listen(Event::Update);
    cortex_m::interrupt::free(|cs| TIM3.borrow(cs).replace(Some(tim3)));

    loop {}
}

#[interrupt]
fn TIM3() {
    static DUTY_STEP: Mutex<RefCell<u16>> = Mutex::new(RefCell::new(0));

    cortex_m::interrupt::free(|cs| {
        let mut step = DUTY_STEP.borrow(cs).borrow_mut();

        let mut tim2_pwm_ch2 = TIM2_PWM_CH2.borrow(cs).borrow_mut();
        let tim2_pwm_ch2 = tim2_pwm_ch2.as_mut();
        let tim2_pwm_ch2 = if let Some(tim2_pwm_ch2) = tim2_pwm_ch2 {
            tim2_pwm_ch2
        } else {
            panic!("not found TIM2 PWM channel 4.");
        };

        let max_duty = tim2_pwm_ch2.get_max_duty();
        let step_val = DUTY_TABLE.get(*step as usize).unwrap_or_else(|| panic!());
        let max_step_val = (DUTY_TABLE.len() / 2) as u16;
        let new_duty = (max_duty / max_step_val) * step_val;
        tim2_pwm_ch2.set_duty(new_duty);

        *step = (*step + 1) % (DUTY_TABLE.len() as u16);

        let mut tim3 = TIM3.borrow(cs).borrow_mut();
        let tim3 = tim3.as_mut();
        let tim3 = if let Some(tim3) = tim3 {
            tim3
        } else {
            panic!("not found TIM3.");
        };
        tim3.clear_interrupt(Event::all());
    })
}
