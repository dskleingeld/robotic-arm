use embassy_nrf::interrupt;
use nrf52832_hal::pac;

use super::{ISR_A, ISR_B, ISR_C};

#[interrupt]
unsafe fn GPIOTE() {
    let g = &*pac::GPIOTE::ptr();

    for i in 0..6 {
        let event = g.events_in[i].read().bits();
        if event != 0 {
            // defmt::debug!("isr: {}", i);
            g.intenclr.write(|w| unsafe { w.bits(1 << i) }); // mark interrupt as handled

            // re-enable interrupts
            g.events_in[i].reset();
            g.intenset.write(|w| unsafe { w.bits(1 << i) });

            // actually do something
            match i {
                0..=1 => ISR_A.update(),
                2..=3 => ISR_B.update(),
                4..=5 => ISR_C.update(),
                _ => defmt::panic!("interrupt on unused channel: {}", i),
            }
        }
    }
}

pub fn enable() {
    use embassy::interrupt::{Interrupt, InterruptExt};
    let ports = unsafe { &[&*pac::P0::ptr()] };

    for &p in ports {
        // Enable latched detection
        p.detectmode.write(|w| w.detectmode().ldetect());
        // Clear latch
        p.latch.write(|w| unsafe { w.bits(0xFFFFFFFF) })
    }

    // Enable interrupts
    let irq = unsafe { interrupt::GPIOTE::steal() };
    irq.unpend();
    irq.set_priority(interrupt::Priority::P7);
    irq.enable();

    let g = unsafe { &*pac::GPIOTE::ptr() };
    g.events_port.write(|w| w);
    g.intenset.write(|w| w.port().set());

    // Set interrupts for encoders
    ISR_A.enable();
    ISR_B.enable();
    ISR_C.enable();
}

pub fn set_pin(pin_numb: u8, channel_numb: usize) {
    defmt::info!("enabling interrupts for pin: {} on channel {}", pin_numb, channel_numb);
   let g = unsafe { &*pac::GPIOTE::ptr() };

    g.config[channel_numb].write(|w| { 
        w.mode().event().polarity().toggle();
        unsafe { w.psel().bits(pin_numb) }
    });

    //enable channel
    g.events_in[channel_numb].reset();
    g.intenset.write(|w| unsafe { w.bits(1 << channel_numb) });
}
