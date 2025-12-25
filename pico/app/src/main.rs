#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use panic_probe as _;

use pico_lib::poro::{
    CarLocation, ParkLocation, Position, Protector, ProtectorMachine, Service, Status,
};

extern crate alloc;

//use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // https://docs.rs/embedded-alloc/latest/embedded_alloc/
    //{
    //    use core::mem::MaybeUninit;
    //    const HEAP_SIZE: usize = 1024;
    //    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    //    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    //}

    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);

    let pm = ProtectorMachine {};
    pm.dump(&Protector {
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

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
