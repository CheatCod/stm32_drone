#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_stm32::time::hz;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

fn set_motor_speed<T: CaptureCompare16bitInstance>(
    pwm: &mut SimplePwm<'_, T>,
    channel: Channel,
    speed: f32,
) {
    let speed = speed.clamp(0.0, 1.0);
    // Assume pwm is 50hz, ESCs are 1ms to 2ms duty cycle
    let period_duration = pwm.get_max_duty() as f32 / 20.0;
    let duty = ((1.0 + speed) * period_duration) as u16;
    pwm.enable(channel);
    pwm.set_duty(channel, duty);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::None);
    let mut button = ExtiInput::new(button, p.EXTI13);

    let servo_pin = PwmPin::new_ch2(p.PB3);
    let mut pwm = SimplePwm::new(p.TIM2, None, Some(servo_pin), None, None, hz(50));

    // wait until button pressed down (is low when pressed)
    info!("waiting for button press");
    button.wait_for_falling_edge().await;
    info!("button pressed, outputting 2ms max duty cycle");
    set_motor_speed(&mut pwm, Channel::Ch2, 1.0);
    button.wait_for_falling_edge().await;
    info!("outputting 1ms min duty cycle");
    set_motor_speed(&mut pwm, Channel::Ch2, 0.0);
}
