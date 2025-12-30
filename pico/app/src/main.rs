#![no_std]
#![no_main]

use atat::asynch::{AtatClient, Client};
use atat::{AtatIngress, DefaultDigester, Ingress, ResponseSlot, UrcChannel};
use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler as AdcInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::peripherals::{UART0, USB};
use embassy_rp::uart::{self, BufferedInterruptHandler, BufferedUart, BufferedUartRx};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use pico_lib::network;
use pico_lib::poro;
use pico_lib::urc::Urc;

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const INGRESS_BUF_SIZE: usize = 1024;
const URC_CAPACITY: usize = 128;
const URC_SUBSCRIBERS: usize = 3;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1280;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    Timer::after(Duration::from_millis(500)).await;
    log::info!("USB Logger Task spawned");

    let pm = poro::ProtectorMachine {};
    let dumped = pm.dump(&poro::Protector {
        car_location: Some(poro::CarLocation {
            position: poro::Position {
                latitude: 46.7624859f64,
                longitude: 18.6304591f64,
            },
            accuracy: 250.25f32,
            battery: 0.8912f32,
            timestamp: 1670077542109i64,
        }),
        park_location: Some(poro::ParkLocation {
            position: poro::Position {
                latitude: 47.1258945f64,
                longitude: 17.8372091f64,
            },
            accuracy: 500.25f32,
        }),
        status: Some(poro::Status::CarTheftDetected),
        service: Some(poro::Service { value: true }),
    });
    log::info!("dumped {}", dumped);

    let mut power = Output::new(p.PIN_14, Level::Low);

    let mut power_on_off = async || {
        log::info!("power on");
        power.set_high();
        Timer::after_secs(2).await;
        power.set_low();
        log::info!("power off");
    };

    power_on_off().await;

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut p26 = Channel::new_pin(p.PIN_26, Pull::None);
    let mut ts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let uart = BufferedUart::new(
        uart,
        tx_pin,
        rx_pin,
        Irqs,
        TX_BUF.init([0; 16]),
        RX_BUF.init([0; 16]),
        uart::Config::default(),
    );
    let (writer, reader) = uart.split();

    static RES_SLOT: ResponseSlot<INGRESS_BUF_SIZE> = ResponseSlot::new();
    static URC_CHANNEL: UrcChannel<Urc, URC_CAPACITY, URC_SUBSCRIBERS> = UrcChannel::new();
    let ingress = Ingress::new(
        DefaultDigester::<Urc>::default(),
        INGRESS_BUF.init([0; INGRESS_BUF_SIZE]),
        &RES_SLOT,
        &URC_CHANNEL,
    );
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let mut client = Client::new(
        writer,
        &RES_SLOT,
        BUF.init([0; 1024]),
        atat::Config::default(),
    );

    Timer::after(Duration::from_millis(500)).await;
    log::info!("Before spawning reader Task");

    spawner.spawn(ingress_task(ingress, reader)).unwrap();

    Timer::after(Duration::from_millis(500)).await;
    log::info!("After spawning reader Task");

    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut counter = 0u8;
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        counter += 1;

        let level = adc.read(&mut p26).await.unwrap();
        let temp = convert_to_celsius(adc.read(&mut ts).await.unwrap());
        log::info!(
            "Tick counter: {} Pin 26 ADC: {} Temp: {}",
            counter,
            level,
            temp
        );

        led.set_low();
        Timer::after(Duration::from_millis(500)).await;

        match counter {
            2 => {
                client.send(&network::ATE {}).await.ok();
            }
            3 => {
                client.send(&network::AT {}).await.ok();
            }
            4 => {
                client.send(&network::CGREG {}).await.ok();
            }
            _ => (),
        }
    }
}

#[embassy_executor::task]
async fn ingress_task(
    mut ingress: Ingress<
        'static,
        DefaultDigester<Urc>,
        Urc,
        INGRESS_BUF_SIZE,
        URC_CAPACITY,
        URC_SUBSCRIBERS,
    >,
    mut reader: BufferedUartRx,
) -> ! {
    log::info!("ingress task spawned...");
    ingress.read_from(&mut reader).await
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Trace, driver);
}

fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as f32) / 10.0
}
