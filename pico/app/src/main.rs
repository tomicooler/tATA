#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use embedded_alloc::LlffHeap as Heap;
use core::ptr::addr_of_mut;

use pico_lib::poro::{
    CarLocation, ParkLocation, Position, Protector, ProtectorMachine, Service, Status,
};

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

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
    let _ = spawner.spawn(logger_task(driver));

    let mut led = Output::new(p.PIN_25, Level::Low);

    let pm = ProtectorMachine {};
    let dumped = pm.dump(&Protector {
        car_location: Some(CarLocation {
            position: Position {
                latitude: 46.7624859f64,
                longitude: 18.6304591f64,
            },
            accuracy: 250.25f32,
            battery: 0.8912f32,
            timestamp: 1670077542109i64,
        }),
        park_location: Some(ParkLocation {
            position: Position {
                latitude: 47.1258945f64,
                longitude: 17.8372091f64,
            },
            accuracy: 500.25f32,
        }),
        status: Some(Status::CarTheftDetected),
        service: Some(Service { value: true }),
    });

    let mut counter = 0;
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        counter += 1;
        log::info!("Tick {} {}", counter, dumped);

        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
