#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let gpout3 = clocks::Gpout::new(p.PIN_25);
    gpout3.set_div(1000, 0);
    gpout3.enable();

    loop {
        gpout3.set_src(clocks::GpoutSrc::CLK_SYS);
        info!(
            "Pin 25 is now outputing CLK_SYS/1000, should be toggling at {}",
            gpout3.get_freq()
        );
        Timer::after(Duration::from_secs(2)).await;

        gpout3.set_src(clocks::GpoutSrc::CLK_REF);
        info!(
            "Pin 25 is now outputing CLK_REF/1000, should be toggling at {}",
            gpout3.get_freq()
        );
        Timer::after(Duration::from_secs(2)).await;
    }
}
