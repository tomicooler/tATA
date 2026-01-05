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
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub;
use embassy_sync::pubsub::Subscriber;
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use pico_lib::poro;
use pico_lib::urc;
use pico_lib::{call, network};

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

    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut power = Output::new(p.PIN_14, Level::Low);

    let mut power_on_off = async || {
        led.set_high();
        log::info!("power on");
        power.set_high();
        Timer::after_secs(2).await;
        power.set_low();
        log::info!("power off");
        led.set_low();
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
    static URC_CHANNEL: UrcChannel<urc::Urc, URC_CAPACITY, URC_SUBSCRIBERS> = UrcChannel::new();
    let ingress = Ingress::new(
        DefaultDigester::<urc::Urc>::default(),
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

    let sub = URC_CHANNEL.subscribe().unwrap();
    spawner.spawn(urc_handler_task(sub)).unwrap();
    log::info!("After spawning Urc Task");
    Timer::after(Duration::from_secs(2)).await;

    log::info!("BEGIN AtSetCommandEchoOff");
    {
        let r = client.send(&network::AtSetCommandEchoOff).await;
        match r {
            Ok(_) => {
                log::info!("AtSetCommandEchoOff response ok");
            }
            Err(e) => log::info!("AtSetCommandEchoOff error: {:?}", e),
        }
    }
    log::info!("END AtSetCommandEchoOff");

    log::info!("BEGIN AtInit");
    loop {
        let r = client.send(&network::AtInit).await;
        match r {
            Ok(_) => {
                log::info!("AtInit response ok");
                break;
            }
            Err(e) => {
                log::info!("AtInit error: {:?}", e);
                power_on_off().await;
            }
        }
    }
    log::info!("END AtInit");

    log::info!("BEGIN AtNetworkRegistrationRead");
    loop {
        log::info!("->1");
        let r = client.send(&network::AtNetworkRegistrationRead).await;
        log::info!("->2");
        // TODO: issue: USB Debug logging stop working after CREG and SMS Ready URC, the program does not crash, led is still blinking
        /*

        Sending command: "AT+CGREG?\r"
        Received response (21/21): "+CGREG: 0,2"
        ->2
        AtNetworkRegistrationRead response ok: Searching
        ->1
        AtNetworkRegistrationRead response ok: Searching
        Received URC/128 (13/13): "SMS Ready"
        URC SMSReady
        ->1
        Sending command: "AT+CGREG?\r"
        Got serial read error Other
        Got serial read error Other
        Received OK (6/6)
        ->2
        AtNetworkRegistrationRead error: Parse

                 */
        match r {
            Ok(n) => {
                log::info!("AtNetworkRegistrationRead response ok: {:?}", n.stat);
                if n.stat == network::NetworkRegistrationStatus::Registered
                    || n.stat == network::NetworkRegistrationStatus::RegisteredRoaming
                {
                    break;
                }
            }
            Err(e) => log::info!("AtNetworkRegistrationRead error: {:?}", e),
        }
        Timer::after(Duration::from_secs(1)).await;
    }
    log::info!("END AtNetworkRegistrationRead");

    log::info!("BEGIN AtSignalQualityReportExecute");
    {
        let r = client.send(&network::AtSignalQualityReportExecute).await;
        match r {
            Ok(n) => {
                log::info!("AtSignalQualityReportExecute response ok: {:?}", n.rssi);
            }
            Err(e) => log::info!("AtSignalQualityReportExecute error: {:?}", e),
        }
    }
    log::info!("END AtSignalQualityReportExecute");

    log::info!("BEGIN AtEnterPinRead");
    {
        let r = client.send(&network::AtEnterPinRead).await;
        match r {
            Ok(n) => {
                log::info!("AtEnterPinRead response ok: {:?}", n.code);
                if n.code != "READY" {
                    led.set_high();
                    log::info!("DISABLE PIN ON SIM CARD!!!");
                    Timer::after(Duration::from_secs(60)).await;
                }
            }
            Err(e) => log::info!("AtEnterPinRead error: {:?}", e),
        }
    }
    log::info!("END AtEnterPinRead");

    log::info!("BEGIN AtSignalQualityReportExecute");
    {
        let r = client.send(&network::AtSignalQualityReportExecute).await;
        match r {
            Ok(n) => {
                log::info!("AtSignalQualityReportExecute response ok: {:?}", n.rssi);
            }
            Err(e) => log::info!("AtSignalQualityReportExecute error: {:?}", e),
        }
    }
    log::info!("END AtSignalQualityReportExecute");

    log::info!("BEGIN AtOperatorSelectionRead");
    {
        let r = client.send(&network::AtOperatorSelectionRead).await;
        match r {
            Ok(n) => {
                log::info!("AtOperatorSelectionRead response ok: {:?}", n.oper);
            }
            Err(e) => log::info!("AtOperatorSelectionRead error: {:?}", e),
        }
    }
    log::info!("END AtOperatorSelectionRead");

    for _ in 0..30 {
        led.set_high();
        Timer::after(Duration::from_millis(50)).await;
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;
    }

    let mut sleeper = Sleeper;
    call::call_number(
        &mut client,
        "+36301234567",
        &mut sleeper,
        Duration::from_secs(6).as_millis(),
    )
    .await;

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
    }
}

#[embassy_executor::task]
async fn ingress_task(
    mut ingress: Ingress<
        'static,
        DefaultDigester<urc::Urc>,
        urc::Urc,
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
async fn urc_handler_task(
    mut sub: Subscriber<
        'static,
        CriticalSectionRawMutex,
        urc::Urc,
        URC_CAPACITY,
        URC_SUBSCRIBERS,
        1,
    >,
) -> ! {
    log::info!("Ucr Handler task spawned...");
    loop {
        let m = sub.next_message().await;
        match m {
            pubsub::WaitResult::Message(u) => match u {
                urc::Urc::CallReady => {
                    log::info!("URC CallReady");
                }
                urc::Urc::SMSReady => {
                    log::info!("URC SMSReady");
                }
            },
            pubsub::WaitResult::Lagged(b) => {
                log::info!("Urc Lagged messages: {}", b);
            }
        }
    }
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

struct Sleeper;
impl call::Sleeper for Sleeper {
    async fn sleep(&mut self, millis: u64) {
        Timer::after(Duration::from_millis(millis)).await
    }
}
