#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::{self, CaptureCompare16bitInstance, Channel};
use embassy_stm32::time::{hz, khz};
use embassy_stm32::usart::{Config, UartTx};
use embassy_time::Delay;
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
    pwm.set_duty(channel, duty);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let button = Input::new(p.PC13, Pull::None);
    let button = ExtiInput::new(button, p.EXTI13);

    let mut led = Output::new(p.PA5, Level::High, Speed::Low);
    let mut delay = Delay;
    let mut adc = Adc::new(p.ADC1, &mut delay);

    let servo_pin = PwmPin::new_ch2(p.PB3);
    let mut pwm = SimplePwm::new(p.TIM2, None, Some(servo_pin), None, None, hz(50));
    pwm.enable(Channel::Ch2);

    loop {
        if button.is_low(){
            set_motor_speed(&mut pwm, Channel::Ch2, 0.0);
            Timer::after(Duration::from_millis(100)).await;
            pwm.disable(Channel::Ch2);
            warn!("button pressed, exiting");
            break;
        }
        led.set_high();
        let sample = adc.read(&mut p.PA0);
        let percent = sample as f32 / 4096.0;
        // servo is 50hz, 20ms period, 1ms to 2ms duty cycle
        set_motor_speed(&mut pwm, Channel::Ch2, percent);
        led.set_low();
        Timer::after(Duration::from_millis(10)).await;
    }
}
