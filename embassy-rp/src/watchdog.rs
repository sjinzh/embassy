//! Watchdog
//!
//! The watchdog is a countdown timer that can restart parts of the chip if it reaches zero. This can be used to restart the
//! processor if software gets stuck in an infinite loop. The programmer must periodically write a value to the watchdog to
//! stop it from reaching zero.
//!
//! Credit: based on `rp-hal` implementation (also licensed Apache+MIT)

use core::marker::PhantomData;

use embassy_time::Duration;

use crate::pac;
use crate::peripherals::WATCHDOG;

/// Watchdog peripheral
pub struct Watchdog {
    phantom: PhantomData<WATCHDOG>,
    load_value: u32, // decremented by 2 per tick (µs)
}

impl Watchdog {
    /// Create a new `Watchdog`
    pub fn new(_watchdog: WATCHDOG) -> Self {
        Self {
            phantom: PhantomData,
            load_value: 0,
        }
    }

    /// Start tick generation on clk_tick which is driven from clk_ref.
    ///
    /// # Arguments
    ///
    /// * `cycles` - Total number of tick cycles before the next tick is generated.
    ///   It is expected to be the frequency in MHz of clk_ref.
    pub fn enable_tick_generation(&mut self, cycles: u8) {
        unsafe {
            let watchdog = pac::WATCHDOG;
            watchdog.tick().write(|w| {
                w.set_enable(true);
                w.set_cycles(cycles.into())
            });
        }
    }

    /// Defines whether or not the watchdog timer should be paused when processor(s) are in debug mode
    /// or when JTAG is accessing bus fabric
    pub fn pause_on_debug(&mut self, pause: bool) {
        unsafe {
            let watchdog = pac::WATCHDOG;
            watchdog.ctrl().write(|w| {
                w.set_pause_dbg0(pause);
                w.set_pause_dbg1(pause);
                w.set_pause_jtag(pause);
            })
        }
    }

    fn load_counter(&self, counter: u32) {
        unsafe {
            let watchdog = pac::WATCHDOG;
            watchdog.load().write_value(pac::watchdog::regs::Load(counter));
        }
    }

    fn enable(&self, bit: bool) {
        unsafe {
            let watchdog = pac::WATCHDOG;
            watchdog.ctrl().write(|w| w.set_enable(bit))
        }
    }

    // Configure which hardware will be reset by the watchdog
    // (everything except ROSC, XOSC)
    unsafe fn configure_wdog_reset_triggers(&self) {
        let psm = pac::PSM;
        psm.wdsel().write_value(pac::psm::regs::Wdsel(
            0x0001ffff & !(0x01 << 0usize) & !(0x01 << 1usize),
        ));
    }

    /// Feed the watchdog timer
    pub fn feed(&mut self) {
        self.load_counter(self.load_value)
    }

    /// Start the watchdog timer
    pub fn start(&mut self, period: Duration) {
        const MAX_PERIOD: u32 = 0xFFFFFF;

        let delay_us = period.as_micros();
        if delay_us > (MAX_PERIOD / 2) as u64 {
            panic!("Period cannot exceed {} microseconds", MAX_PERIOD / 2);
        }
        let delay_us = delay_us as u32;

        // Due to a logic error, the watchdog decrements by 2 and
        // the load value must be compensated; see RP2040-E1
        self.load_value = delay_us * 2;

        self.enable(false);
        unsafe {
            self.configure_wdog_reset_triggers();
        }
        self.load_counter(self.load_value);
        self.enable(true);
    }
}
