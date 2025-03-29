#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Triangle};
use embedded_graphics::text::Text;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::prelude::*;
use embedded_graphics::Drawable;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::Builder;
use panic_rtt_target as _;

extern crate alloc;

const W: u16 = 135; //display width
const H: u16 = 240; //display height

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.1
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    // TODO: Spawn some tasks
    let _ = spawner;

    //{'BTN_A': 2,
    //'BTN_B': 1,
    //'BTN_FRONT': 6,
    //'JUP': 15,
    //'JDOWN': 16,
    //'JLEFT': 8,
    //'JRIGHT': 7,
    //'JCEN': 9,
    //'G0': 18, 'G1': 17, 'G2': 4, 'G3': 5,
    // 'LED_DATA': 46,
    //'LED_PWREN': 3,
    //'LCD_PWR': 39,
    //'LCD_BLEN': 0,
    //'SCL_S': 41,
    //'SDA_S': 42, 'SCL_P': 43, 'SDA_P': 44,
    //'LCD_CS': 10, 'LCD_CLK': 12, 'LCD_DIN': 11, 'LCD_RESET': 14, 'LCD_DC': 13,
    // 'UVLO_TRIG': 45, 'ACCEL_INT': 40, 'CHARGE_DET': 21}
    //
    let mut cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default()); //chip select
    let sclk = peripherals.GPIO12; //clock


    //From: https://github.com/emfcamp/TiDAL-Firmware/blob/main/modules/tidal.py
    //_LCD_SPI = SPI(2, baudrate=40000000, polarity=0, sck=_LCD_CLK, mosi=_LCD_DIN)
// _LCD_DC = Pin(_hw["LCD_DC"], Pin.OUT)
// display = st7789.ST7789(_LCD_SPI, 135, 240, cs=_LCD_CS, reset=_LCD_RESET, dc=_LCD_DC, rotation=2)

    let mut dc = peripherals.GPIO13; // Assuming this is LCD_DC
    let mosi = peripherals.GPIO11;
    let rst = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let mut backlight = Output::new(peripherals.GPIO0, Level::High, OutputConfig::default()); //Low is on, High is off?
  

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_mosi(mosi)
    .with_sck(sclk).into_async();

    let mut buffer = [0_u8; 512];

    let dc_out = Output::new(&mut dc, Level::High, OutputConfig::default());
    //let rst_out = Output::new(rst, Level::High, OutputConfig::default());


    let dev = ExclusiveDevice::new_no_delay(spi, &mut cs).unwrap();
    let di = SpiInterface::new(dev, dc_out, &mut buffer);

    let mut delay = Delay::new();
    let mut display = Builder::new(ST7789, di)
        .display_size(W, H)
        //.invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

   

    backlight.toggle();
  
    loop {
        let  delay = Delay::new();
        delay.delay_millis(1000);
        draw_text(&mut display).unwrap();
        delay.delay_millis(1000);
        display.clear(Rgb565::BLACK).unwrap();
        backlight.toggle();
    }

}

fn draw_text<T: DrawTarget<Color = Rgb565>>(display: &mut T) -> Result<(), T::Error> {

  
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let text = "Hello World ^_^;";

    Text::new(text, Point::new(50, 50), text_style)
    .draw(display)?;

    Ok(())
}
