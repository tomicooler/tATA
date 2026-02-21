#![no_std]
#![no_main]

use atat::asynch::Client;
use atat::heapless::String;
use atat::{AtatIngress, DefaultDigester, Ingress, ResponseSlot, UrcChannel};
use core::ptr::addr_of_mut;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler as AdcInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::peripherals::{RTC, UART0};
use embassy_rp::rtc::{DateTime, DateTimeFilter, DayOfWeek, Rtc};
use embassy_rp::uart::{self, BufferedInterruptHandler, BufferedUart, BufferedUartRx};
use embassy_sync::pubsub;
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
use pico_lib::service::{Configuration, DeviceStatus, Service};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::watchdog::Watchdog;
use pico_lib::at;
use pico_lib::at::PicoHW;
use pico_lib::urc;

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const INGRESS_BUF_SIZE: usize = 1024;
const URC_CAPACITY: usize = 128;
const URC_SUBSCRIBERS: usize = 3;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
    RTC_IRQ => embassy_rp::rtc::InterruptHandler;
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

    let watchdog = Watchdog::new(p.WATCHDOG);
    spawner.spawn(watchdog_task(watchdog)).unwrap();

    let mut pico = Pico {
        led: Output::new(p.PIN_25, Level::Low),
        power: Output::new(p.PIN_14, Level::Low),
        rtc: Rtc::new(p.RTC, Irqs),
    };

    if !pico.rtc.is_running() {
        let now = DateTime {
            year: 2000,
            month: 1,
            day: 1,
            day_of_week: DayOfWeek::Saturday,
            hour: 0,
            minute: 0,
            second: 0,
        };
        pico.rtc.set_datetime(now).unwrap();
        // The rp2040 chip will always add a Feb 29th on every year that is divisible by 4,
        // but this may be incorrect (e.g. on century years)
        pico.rtc.set_leap_year_check(false);
    }

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut p26 = Channel::new_pin(p.PIN_26, Pull::None);
    let mut ts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static TX_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    let uart = BufferedUart::new(
        uart,
        tx_pin,
        rx_pin,
        Irqs,
        TX_BUF.init([0; 512]),
        RX_BUF.init([0; 512]),
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
    static BUF: StaticCell<[u8; 2048]> = StaticCell::new();
    let mut client = Client::new(
        writer,
        &RES_SLOT,
        BUF.init([0; 2048]),
        atat::Config::default(),
    );

    spawner.spawn(ingress_task(ingress, reader)).unwrap();

    let mut sub = URC_CHANNEL.subscribe().unwrap();

    let mut service = Service {
        cfg: Configuration {
            phone_number: String::try_from("+36301234567").unwrap(),
            sms_password: String::try_from("12345").unwrap(),
            service_enabled: true,
            locator_poll_count: 10,
            check_period_seconds: 15 * 60,
            call_after_boot: true,
            debug_alerts: true,
            battery_alerts: true,
            detect_parking: true,
            keep_n_sms: 20,
        },
        status: DeviceStatus {
            last_big_location_change: 0,
            location: None,
            park_location: None,
            battery: 100.0f32,
            last_battery_alert: 0,
        },
    };

    service.init(&mut client, &mut pico).await;

    let mut counter = 0u64;
    pico.rtc
        .schedule_alarm(DateTimeFilter::default().second(30));

    loop {
        match select3(
            Timer::after_secs(30),
            pico.rtc.wait_for_alarm(),
            sub.next_message(),
        )
        .await
        {
            // Timer expired
            Either3::First(_) => {
                pico.set_led_high();
                Timer::after(Duration::from_millis(500)).await;
                let dt = pico.rtc.now().unwrap();
                info!(
                    "Now: {}-{:02}-{:02} {}:{:02}:{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                );

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
            // Alarm triggered
            Either3::Second(_) => {
                let dt = pico.rtc.now().unwrap();
                info!(
                    "ALARM TRIGGERED! Now: {}-{:02}-{:02} {}:{:02}:{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                );

                service.refresh(&mut client, &mut pico).await;

                // every 10 minute, todo..
                pico.rtc
                    .schedule_alarm(DateTimeFilter::default().minute((dt.minute + 10) % 60));
            }
            // Unsolicited Message
            Either3::Third(m) => match &m {
                pubsub::WaitResult::Message(u) => match u {
                    urc::Urc::ClipUrc(v) => {
                        service
                            .handle_incoming_call(&mut client, &mut pico, &v.number)
                            .await;
                    }
                    urc::Urc::NewMessageIndicationUrc(v) => {
                        service
                            .handle_sms(&mut client, &mut pico, v.index as u32)
                            .await;
                    }
                    _ => (),
                },
                pubsub::WaitResult::Lagged(b) => {
                    info!("Urc Lagged messages: {}", b);
                }
            },
        }
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
async fn watchdog_task(mut watchdog: Watchdog) -> ! {
    info!("WATCHDOG TASK SPAWNED");
    watchdog.start(Duration::from_millis(6000));
    loop {
        Timer::after_millis(2000).await;
        debug!("    WATCHDOG FEED");
        watchdog.feed();
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
    rtc: Rtc<'a, RTC>,
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
        // 5.3.2.3. Restart GSM by PWRKEY
        //   1. power off the GSM (between 1.5s and 2s)
        //   2. wait 800ms
        //   3. power on the GSM (> 800ms)

        info!("Sim868 restart procedure power");
        info!(
            "  power: is_high? {} is low? {}",
            self.power.is_set_high(),
            self.power.is_set_low()
        );

        /*
        This power off procedure is not working as expected. TODO: investigate more.

        self.led.set_high();
        info!("Sim868 power off");
        self.power.set_high();
        info!("  power: is_high? {} is low? {}", self.power.is_set_high(), self.power.is_set_low());
        // Customer can power off GSM by pulling down the PWRKEY pin for at least 1.5 second and release.
        Timer::after_millis(1600).await;
        self.power.set_low();
        self.led.set_low();
        info!("  power: is_high? {} is low? {}", self.power.is_set_high(), self.power.is_set_low());
        */

        Timer::after_secs(1).await;
        self.led.set_high();
        info!("Sim868 power on");
        self.power.set_high();
        info!(
            "  power: is_high? {} is low? {}",
            self.power.is_set_high(),
            self.power.is_set_low()
        );
        // Customer can power on GSM by pulling down the PWRKEY pin for at least 1 second and then release.
        Timer::after_millis(900).await;
        self.power.set_low();
        self.led.set_low();
        info!(
            "  power: is_high? {} is low? {}",
            self.power.is_set_high(),
            self.power.is_set_low()
        );

        Timer::after_secs(2).await;
        info!("Sim868 should be Ready");

        // Power off GSM by AT command “AT+CPOWD=1”.

        // The GSM will restart after pulling the PWRKEY over 33 seconds.

        // Customer can use AT command “AT+IPR=x” to set a fixed baud rate and save the configuration to
        // non-volatile flash memory. After the configuration is saved as fixed baud rate, the Code “RDY” should be
        // received from the serial port every time when SIM868 is powered on. For details, please refer to the chapter
        // “AT+IPR” in document [1]
    }

    fn uptime_millis(&mut self) -> i64 {
        let dt = self.rtc.now().unwrap();
        let now = fasttime::DateTime {
            date: fasttime::Date {
                year: dt.year as i32,
                month: dt.month,
                day: dt.day,
            },
            time: fasttime::Time {
                hour: dt.hour,
                minute: dt.minute,
                second: dt.second,
                nanosecond: 0,
            },
        };
        return (now.unix_timestamp_nanos() / 1_000_000) as i64;
    }
}
