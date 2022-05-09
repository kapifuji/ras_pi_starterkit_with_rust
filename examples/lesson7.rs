// スイッチを押すとLEDのON/OFFを切り替える。

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

use stm32f4xx_hal as hal;

use hal::{
    gpio::{Input, Output, PinState, PullUp, PushPull, PB13, PC8},
    pac,
    prelude::*,
};

static LED: Mutex<RefCell<Option<PB13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static SWITCH: Mutex<RefCell<Option<PC8<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut peripheral = pac::Peripherals::take().unwrap();

    // クロック設定
    let rcc = peripheral.RCC.constrain();
    let _clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .bypass_hse_oscillator()
        .sysclk(180.MHz())
        .pclk1(45.MHz()) // peripheral clock 1
        .pclk2(90.MHz()) // peripheral clock 2
        .freeze();

    // LED 出力用 GPIO 設定
    let gpiob = peripheral.GPIOB.split();
    let pb13 = gpiob.pb13.into_push_pull_output_in_state(PinState::High);

    // Switch 入力用 GPIO 設定
    let mut syscfg = peripheral.SYSCFG.constrain();
    let gpioc = peripheral.GPIOC.split();
    let mut pc8 = gpioc.pc8.into_pull_up_input();
    pc8.make_interrupt_source(&mut syscfg);
    pc8.enable_interrupt(&mut peripheral.EXTI);
    pc8.trigger_on_edge(&mut peripheral.EXTI, hal::gpio::Edge::Falling);

    cortex_m::interrupt::free(|cs| {
        LED.borrow(cs).replace(Some(pb13));
        SWITCH.borrow(cs).replace(Some(pc8));
    });

    // 割り込み有効化
    unsafe {
        // EXTI9_5割り込み有効化(EXTI8が目的)
        pac::NVIC::unmask(stm32f446::Interrupt::EXTI9_5);
    }

    loop {}
}

#[interrupt]
fn EXTI9_5() {
    cortex_m::interrupt::free(|cs| {
        let mut led = LED.borrow(cs).borrow_mut();
        let led = led.as_mut();
        let led = if let Some(led) = led {
            led
        } else {
            panic!("LED が見つかりませんでした。");
        };

        let mut switch = SWITCH.borrow(cs).borrow_mut();
        let switch = switch.as_mut();
        let switch = if let Some(switch) = switch {
            switch
        } else {
            panic!("SWITCH が見つかりませんでした。");
        };

        led.toggle();

        // 簡易チャタリング対策
        // もっと真面目にやるなら、割り込みではなく一定msおきに複数回サンプリングして連続してON/OFF状態が継続で確定とする。
        // そして、OFFからONのときのみ押下されたとして、LEDをトグルする。
        // 他の方法もあるはず。また、回路側での対策もある。
        wait_ms(5);

        switch.clear_interrupt_pending_bit();
    })
}

// 指定した時間待ちます。（待ち時間の精度は悪いので注意）
fn wait_ms(msec: u16) {
    for _ in 0..msec {
        // nopが180000回で1ms（本当はforではなく、可能な限り列挙した方が誤差少ない）
        for _ in 0..18000 {
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
        }
    }
}
