# the Anti Theft Auto

This is tATA on Raspberry [Pico Sim868-GSM/GPRS/GNS](https://www.waveshare.com/wiki/Pico-SIM868-GSM/GPRS/GNSS).

## What is tATA?

[![tATA video](https://img.youtube.com/vi/vHlMRs05CKg/0.jpg)](https://www.youtube.com/watch?v=vHlMRs05CKg)

^ check the video.

Features:

- retrieve car location
- location based car theft detection
- spy on microphone
- low battery alert

This is a remake of my old android based application ([tATA Watcher SMS on Google Play](https://play.google.com/store/apps/details?id=com.tomicooler.tata.watchersms)). The goal is to re-create tATA Protector on Raspberry Pico.

A pre-built apk can be downloaded from here: [WatcherSMS](https://github.com/tomicooler/tATAPowerDetector/raw/master/releases/watchersms.apk).

NOTE. Work in Progress:

- learning Rust on the way
- pico app does nothing yet
- cpp version with some Sim868 interfacing can be found on the [cpp-backup](https://github.com/tomicooler/tATA/tree/cpp-backup) branch

## Project structure

- app: the tATA Watcher SMS android application.
- pico: the tATA Protector application for Raspberry RP2040 micro controller written in Rust.

## SMS commands

```
$tATA/location/12345
$tATA/call/12345
$tATA/park [on/off]/12345
$tATA/service [on/off]/12345
```

TODO: configuration by SMS commands.

## Development

Building the uf2 file:

```shell
cd pico/app
cargo run
```

Running the tests, etc:

```shell
cd pico/pico-lib
cargo test
cargo check
cargo fmt
# run a specific test with env logger
RUST_LOG=debug cargo test test_call_number -- --nocapture
```

Reading the logs:

```shell
minicom -b 115200 -o -D /dev/serial0 # for uart
minicom -b 115200 -o -D /dev/ttyACM0 # for usb
```
