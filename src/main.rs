#![no_std]
#![no_main]

use esp32s3_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc, gpio::IO, spi::{Spi, SpiMode}, Delay};
use esp_backtrace as _;
use xtensa_lx_rt::entry;
use mipidsi::{Display, DisplayOptions, Orientation};
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Text, TextStyle, TextStyleBuilder};

use profont::{PROFONT_24_POINT};

const TEXT_STYLE : TextStyle = TextStyleBuilder::new()
    .alignment(embedded_graphics::text::Alignment::Center)
    .baseline(embedded_graphics::text::Baseline::Middle)
    .build();

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let sclk = io.pins.gpio7;
    let mosi = io.pins.gpio6;

    let spi = Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        sclk,
        mosi,
        4u32.MHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let dc = io.pins.gpio4.into_push_pull_output();
    let rst = io.pins.gpio48.into_push_pull_output();

    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut display = Display::ili9342c_rgb565(di, rst);

    display
        .init(
            &mut delay,
            DisplayOptions {
                orientation: Orientation::PortraitInverted(false),
                ..DisplayOptions::default()
            },
        )
        .unwrap();

    let mut backlight = io.pins.gpio45.into_push_pull_output();
    backlight.set_high().unwrap();

    Text::with_text_style(
            "Hello From Rust!",
            display.bounding_box().center() - Size::new(0, 25),
            MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::WHITE),
            TEXT_STYLE,
        )
        .draw(&mut display).unwrap();

    loop {
    }
}
