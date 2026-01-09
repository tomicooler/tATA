#![no_std]
#![no_main]

use alloc::string::ToString;
use atat::asynch::Client;
use atat::{AtatIngress, DefaultDigester, Ingress, ResponseSlot, UrcChannel};
use core::ptr::addr_of_mut;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler as AdcInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{self, BufferedInterruptHandler, BufferedUart, BufferedUartRx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub;
use embassy_sync::pubsub::Subscriber;
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use pico_lib::at::PicoHW;
use pico_lib::poro;
use pico_lib::urc;
use pico_lib::utils::send_command_logged;
use pico_lib::{at, battery, call, gps, gsm, network, sms};

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const INGRESS_BUF_SIZE: usize = 1024;
const URC_CAPACITY: usize = 128;
const URC_SUBSCRIBERS: usize = 3;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("STARTING");
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 4096;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }
    let p = embassy_rp::init(Default::default());
    Timer::after(Duration::from_secs(2)).await;
    info!("STARTED");

    let mut pico = Pico {
        led: Output::new(p.PIN_25, Level::Low),
        power: Output::new(p.PIN_14, Level::Low),
    };

    // This is just a Test will be removed later.
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
    info!("PORO TEST: {}", dumped.as_str());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut p26 = Channel::new_pin(p.PIN_26, Pull::None);
    let mut ts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    let uart = BufferedUart::new(
        uart,
        tx_pin,
        rx_pin,
        Irqs,
        TX_BUF.init([0; 256]),
        RX_BUF.init([0; 256]),
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
    info!("Before spawning reader Task");

    spawner.spawn(ingress_task(ingress, reader)).unwrap();

    Timer::after(Duration::from_millis(500)).await;
    info!("After spawning reader Task");

    let sub = URC_CHANNEL.subscribe().unwrap();
    spawner.spawn(urc_handler_task(sub)).unwrap();
    info!("After spawning Urc Task");
    Timer::after(Duration::from_secs(2)).await;

    pico.restart_module().await;
    info!("Network init");
    Timer::after(Duration::from_secs(2)).await;

    network::init_network(&mut client, &mut pico).await;
    sms::init(&mut client, &mut pico).await;
    call::init(&mut client, &mut pico).await;

    for _ in 0..30 {
        pico.set_led_high();
        Timer::after(Duration::from_millis(100)).await;
        pico.set_led_low();
        Timer::after(Duration::from_millis(100)).await;
    }

    match send_command_logged(
        &mut client,
        &battery::AtBatteryChargeExecute,
        "AtBatteryChargeExecute".to_string(),
    )
    .await
    {
        Ok(v) => info!("  {:?}", v),
        Err(_) => (),
    }

    match gps::get_gps_location(&mut client, &mut pico, 10).await {
        Some(v) => info!("GPS location: {:?}", v),
        None => (),
    }

    match gsm::get_gsm_location(&mut client, &mut pico, 10, "online").await {
        Some(v) => info!("GSM location: {:?}", v),
        None => (),
    }

    const PHONE_NUMBER: &'static str = "+36301234567";

    call::call_number(
        &mut client,
        &mut pico,
        PHONE_NUMBER,
        Duration::from_secs(10).as_millis(),
    )
    .await;

    sms::send_sms(
        &mut client,
        &mut pico,
        PHONE_NUMBER,
        "this is a text message",
    )
    .await;

    sms::receive_sms(&mut client, &mut pico).await;

    let mut counter = 0u64;
    loop {
        pico.set_led_high();
        Timer::after(Duration::from_millis(500)).await;

        counter += 1;

        let level = adc.read(&mut p26).await.unwrap();
        let temp = convert_to_celsius(adc.read(&mut ts).await.unwrap());
        info!(
            "Tick counter: {} Pin 26 ADC: {} Temp: {}",
            counter, level, temp
        );

        pico.set_led_low();
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
    info!("INGRESS TASK SPAWNED");
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
    info!("URC TASK SPAWNED");
    loop {
        let m = sub.next_message().await;
        match m {
            pubsub::WaitResult::Message(u) => match u {
                urc::Urc::CallReady => {
                    info!("URC CallReady");
                }
                urc::Urc::SMSReady => {
                    info!("URC SMSReady");
                }
                urc::Urc::SetBearer(_v) => {
                    info!("URC SetBearer");
                }
                urc::Urc::GprsDisconnected(_v) => {
                    info!("URC GprsDisconnected");
                }
                urc::Urc::Ring => {
                    info!("URC Ring");
                }
                urc::Urc::NormalPowerDown => {
                    info!("URC NormalPowerDown");
                }
                urc::Urc::UnderVoltagePowerDown => {
                    info!("URC UnderVoltagePowerDown");
                }
                urc::Urc::UnderVoltageWarning => {
                    info!("URC UnderVoltageWarning");
                }
                urc::Urc::OverVoltagePowerDown => {
                    info!("URC OverVoltagePowerDown");
                }
                urc::Urc::OverVoltageWarning => {
                    info!("URC OverVoltageWarning");
                }
                urc::Urc::ChargeOnlyMode => {
                    info!("URC ChargeOnlyMode");
                }
                urc::Urc::Ready => {
                    info!("URC Ready");
                }
                urc::Urc::ConnectOK1 => {
                    info!("URC ConnectOK1");
                }
                urc::Urc::ConnectOK => {
                    info!("URC ConnectOK");
                }
                urc::Urc::ClipUrc(v) => {
                    info!("URC ClipUrc number={}, type={}", v.number, v.type_);
                }
                urc::Urc::NewMessageIndicationUrc(v) => {
                    info!(
                        "URC NewMessageIndicationUrc index={} mem={}",
                        v.index, v.mem
                    );
                }
            },
            pubsub::WaitResult::Lagged(b) => {
                info!("Urc Lagged messages: {}", b);
            }
        }
    }
}

fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as f32) / 10.0
}

struct Pico<'a> {
    led: Output<'a>,
    power: Output<'a>,
}

impl at::PicoHW for Pico<'_> {
    async fn sleep(&mut self, millis: u64) {
        Timer::after(Duration::from_millis(millis)).await
    }

    fn set_led_high(&mut self) {
        self.led.set_high();
    }

    fn set_led_low(&mut self) {
        self.led.set_low();
    }

    async fn restart_module(&mut self) {
        info!("Sim868 restart procedure");

        self.led.set_high();
        info!("Sim868 power off");
        self.power.set_high();
        // Customer can power off GSM by pulling down the PWRKEY pin for at least 1.5 second and release.
        Timer::after_secs(2).await;
        self.power.set_low();
        self.led.set_low();

        Timer::after_secs(1).await;
        self.led.set_high();
        info!("Sim868 power on");
        self.power.set_high();
        // Customer can power on GSM by pulling down the PWRKEY pin for at least 1 second and then release.
        Timer::after_secs(1).await;
        self.power.set_low();
        self.led.set_low();

        Timer::after_secs(2).await;
        info!("Sim868 should be Ready");

        // Power off GSM by AT command “AT+CPOWD=1”.

        // The GSM will restart after pulling the PWRKEY over 33 seconds.

        // Customer can use AT command “AT+IPR=x” to set a fixed baud rate and save the configuration to
        // non-volatile flash memory. After the configuration is saved as fixed baud rate, the Code “RDY” should be
        // received from the serial port every time when SIM868 is powered on. For details, please refer to the chapter
        // “AT+IPR” in document [1]
    }
}
