#include "sim868.h"

#include <ctype.h>
#include <hardware/uart.h>
#include <hardware/adc.h>
#include <pico/stdlib.h>
#include <stdio.h>
#include <stdlib.h>

namespace Sim868 {
static constexpr uint POWER_PIN = 14;
static constexpr uint LED_PIN = 25;
static constexpr uint BAUD_RATE = 115200;
static constexpr uint UART_TX_PIN0 = 0;
static constexpr uint UART_RX_PIN0 = 1;
} // namespace Sim868

void Sim868::init() {
  sleep_ms(3000);

  printf(">Sim868::init\n");
  auto gpio_init_mode = [](uint pin, uint mode) {
    gpio_init(pin);
    gpio_set_dir(pin, mode);
  };

  gpio_init_mode(POWER_PIN, 1);//GPIO_OUT
  gpio_init_mode(LED_PIN, 1);
  gpio_put(POWER_PIN, 1);
  gpio_put(LED_PIN, 0);

  sleep_ms(3000);

  uart_init(uart0, BAUD_RATE);
  gpio_set_function(UART_TX_PIN0, GPIO_FUNC_UART);
  gpio_set_function(UART_RX_PIN0, GPIO_FUNC_UART);

  adc_init();
  adc_gpio_init(26);
  adc_set_temp_sensor_enabled(true);

  printf("<Sim868::init\n");
}

void Sim868::powerOnOff() {
  printf(">Sim868::power\n");

  gpio_put(POWER_PIN, 1);
  sleep_ms(2000);
  gpio_put(POWER_PIN, 0);

  printf("<Sim868::power\n");
}

void Sim868::ledBlink() {
  printf(">Sim868::led_blink\n");

  for (int i = 1; i <= 10; ++i) {
    gpio_put(LED_PIN, (i % 2));
    sleep_ms(250);
  }

  printf("<Sim868::led_blink\n");
}

void Sim868::start() {
  printf(">Sim868::start\n");
  while (true) {
    sendCommand("ATE1");
    sleep_ms(2000);
    if (const auto resp = sendCommand("AT");
        resp.find("OK") != std::string::npos) {
      printf("SIM868 is ready\r\n");
      break;
    } else {
      powerOnOff();
      printf("SIM868 is starting up, please wait...\r\n");
      sleep_ms(2000);
    }
  }
  printf(">Sim868::end\n");
}

std::string Sim868::sendCommand(const std::string &command,
                                const uint64_t timeout) {
  printf(">Sim868::sendCommand\n");
  printf("  Command: '%s'\n", command.c_str());

  auto cmd = command + "\r\n";
  uart_puts(uart0, cmd.c_str());

  std::string response;
  const uint64_t now = time_us_64();
  while (time_us_64() - now < timeout) {
    while (uart_is_readable_within_us(uart0, 2000)) {
      response += uart_getc(uart0);
    }
  }

  printf("  Response: '%s'\n", response.c_str());
  printf("<Sim868::sendCommand\n");
  sleep_ms(1000);
  return response;
}
