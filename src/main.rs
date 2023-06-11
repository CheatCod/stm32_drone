#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Speed, Input, Pull};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::{self, Channel};
use embassy_stm32::time::{hz, khz};
use embassy_stm32::usart::{Config, UartTx};
use embassy_time::Delay;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let button = Input::new(p.PC13, Pull::None);
    let mut button = ExtiInput::new(button, p.EXTI13);

    let mut led = Output::new(p.PA5, Level::High, Speed::Low);
    let mut delay = Delay;
    let mut adc = Adc::new(p.ADC1, &mut delay);

    let servo_pin = PwmPin::new_ch2(p.PB3);
    let mut pwm = SimplePwm::new(p.TIM2, None, Some(servo_pin), None, None, hz(50));
    let max_duty = pwm.get_max_duty();

    let mut button_last_pressed = button.is_low();
    loop {
        let button_pressed = button.is_low();

        if(button_last_pressed && !button_pressed) {
            pwm.disable(Channel::Ch2);
        }
        if(!button_last_pressed && button_pressed) {
            pwm.enable(Channel::Ch2);
        }
        button_last_pressed = button_pressed;

        led.set_high();
        let sample = adc.read(&mut p.PA0);
        let percent = sample as f32 / 4096.0 / 2.0;
        // servo is 50hz, 20ms period, 1ms to 2ms duty cycle
        let period_duration = max_duty as f32 / 20.0;
        let duty = ((1.0 + percent) * period_duration) as u16;

        pwm.set_duty(
            Channel::Ch2,
            duty,
        );

        info!("outputting {}/{} ({}%)", duty, max_duty, percent * 100.0);
        info!("input: {}", button_pressed);
        led.set_low();
        Timer::after(Duration::from_millis(10)).await;
    }
}
