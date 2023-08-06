#include <pico.h>
#include <pico/stdio.h>

#include <pico/stdlib.h>

#include <vector>

#include "sim868.h"
#include "phonenum.h"

struct Command {
  std::string command;
  std::string response;
};

namespace Commands {

constexpr char APN[] = "online";
constexpr char OK[] = "OK";

Command checkNetworkStatus{"AT+CGREG?", "0,1"};
Command checkPIN{"AT+CPIN?", OK};
Command reportSignalQuality{"AT+CSQ", OK};    // +CSQ: 19,0
Command checkNetworkOperator{"AT+COPS?", OK}; // +COPS: 0,0,"PANNON GSM"

Command checkGPRSService{"AT+CGATT?", OK}; // +CGATT: 1
Command attachGPRSService{"AT+CGATT=1", OK};
Command dettachGPRSService{"AT+CGATT=0", OK};

Command startTaskAPN{std::string{"AT+CSTT=\""} + APN + "\"", OK};
Command bringUpGPRSorCSD{"AT+CIICR", OK};
Command getLocalIPAddress{"AT+CIFSR", OK}; // 100.124.173.143

// AT+CEGPRS=
Command switchEdgeON{"AT+CEGPRS=1", OK};
Command siwtchEdgeOFF{"AT+CEGPRS=0", OK};

Command setBearerGPRS{"AT+SAPBR=3,1,\"Contype\",\"GPRS\"", OK};
Command setBearerAPN{std::string{"AT+SAPBR=3,1,\"APN\",\""} + APN + "\"", OK};
Command activateBearer{"AT+SAPBR=1,1", OK};
Command readBearer{"AT+SAPBR=2,1", OK};

Command getCustomerID{"AT+CLBSCFG=0,1", OK};
Command getUsedTimes{"AT+CLBSCFG=0,2", OK};
Command getLBSAddress{
    "AT+CLBSCFG=0,3",
    OK}; // +CLBSCFG: 0,3,"lbs-simcom.com:3002"    3002 is free!
Command setLBSAddressFree{"AT+CLBSCFG=1,3,\"lbs-simcom.com:3002\"", OK};

Command getLocationLatLong{"AT+CLBS=1,1", OK};
Command getLocationAccessTimes{"AT+CLBS=3,1", OK};
Command getLocationLatLongPrecisionDateTime{"AT+CLBS=4,1", OK};

Command deactivateBearer{"AT+SAPBR=0,1", OK};

Command gpsPowerON{"AT+CGNSPWR=1", OK};
Command gpsPowerOFF{"AT+CGNSPWR=0", OK};
Command gpsGetInfo{
    "AT+CGNSINF",
    OK}; // +CGNSINF:
         // 1,1,20221212120221.000,46.7624859,18.6304591,329.218,2.20,285.8,1,,2.1,2.3,0.9,,7,6,,,51,,

Command setAUXAudio{"AT+CHFA=1", OK};
Command callNumber{std::string{"ATD"} + phoneNum + ";", OK};
Command hangup{"AT+CHUP;", OK};

Command setSMSTextMode{"AT+CMGF=1", OK};
Command sendSMSCommand{std::string{"AT+CMGS=\""} + phoneNum + "\"", ">"};

Command checkBattery{"AT+CBC", "OK"}; // +CBC: <bcs>,<bcl>,<voltage>    0 notcharging 1 charging 2 charging finished, 1..100, mV

} // namespace Commands

void checkNetwork() {
  using namespace Commands;

  for (int i = 0; i < 3; ++i) {
    const auto resp = Sim868::sendCommand(checkNetworkStatus.command);
    if (resp.find(checkNetworkStatus.response) != std::string::npos) {
      break;
    } else {
      printf("checknetwork failed\n");
      sleep_ms(2000);
    }
  }

  std::vector<Command> commands{
      checkPIN,     reportSignalQuality, checkNetworkOperator, checkGPRSService,
      startTaskAPN, bringUpGPRSorCSD,    getLocalIPAddress};

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }
}

void gpsLocation() {
  using namespace Commands;

  std::vector<Command> commands{
      gpsPowerON, gpsGetInfo, gpsGetInfo, gpsGetInfo, gpsGetInfo, gpsGetInfo,
      gpsGetInfo, gpsGetInfo, gpsGetInfo, gpsGetInfo, gpsGetInfo, gpsPowerOFF};

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }

  // todo wait: if (sendCMD_waitResp("AT+CGNSINF", ",,,,", 2000) == 1)
  //{
  //     printf("GPS is not ready\r\n");
}

void gsmLocation() {
  using namespace Commands;

  // https://www.re-innovation.co.uk/docs/find-location-with-sim800l/
  std::vector<Command> commands{setBearerGPRS,
                                setBearerAPN,
                                activateBearer,
                                readBearer,
                                getCustomerID,
                                getUsedTimes,
                                getLBSAddress,
                                setLBSAddressFree,
                                getLocationLatLong,
                                getLocationAccessTimes,
                                getLocationLatLongPrecisionDateTime,
                                getLocationLatLong,
                                getLocationAccessTimes,
                                getLocationLatLongPrecisionDateTime,
                                getLocationLatLong,
                                getLocationAccessTimes,
                                getLocationLatLongPrecisionDateTime,
                                deactivateBearer};

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }
}

void startCall() {
  using namespace Commands;
  using namespace Commands;

  std::vector<Command> commands{setAUXAudio, callNumber};

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }
  // todo how to wait for call to end

  sleep_ms(3*1000);
  Sim868::sendCommand(hangup.command);
}

#include <hardware/uart.h>

void sendSMS() {
  using namespace Commands;
  std::vector<Command> commands{setSMSTextMode, sendSMSCommand};

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }

  std::string msg = std::string{"hello world!"} + std::string{'\x1A'};
  uart_puts(uart0, msg.c_str());
}

void getBattery() {
  // Detect charge:
  // https://forums.raspberrypi.com/viewtopic.php?t=301403
  // There's a signal from USB input that goes to GPIO24 via a potential divider (to create 3.3V logic levels). Looks as if it should read high for USB power, low for 'other' power.
  using namespace Commands;
  std::vector<Command> commands{checkBattery};

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }
}

namespace Commands{
//Command setSMSTextMode{"AT+CMGF=1", OK};
Command checkSMSStorage{"AT+CMGL=?", OK};
Command listSMSAll{"AT+CMGL=\"ALL\"", OK};
//Command setSMSTextMode{"AT+CMGF=1", OK};
//Command setSMSTextMode{"AT+CMGF=1", OK};
}

// Sleep related stuff:
// https://github.com/Xinyuan-LilyGO/LilyGo-T-Call-SIM800/issues/97
// https://docs.rs-online.com/2190/0900766b815ca94b.pdf
void checkSMS() {
  using namespace Commands;

  std::vector<Command> commands{
                                setSMSTextMode,
      Command{"AT+CSCS=\"GSM\"", "OK"},
   //   Command{"AT+CMGS=\"+3620xxxxxxx\"", ">"}, // send sms to myself
      };

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }

 // std::string msg = std::string{"hello world!"} + std::string{'\x1A'};
 // uart_puts(uart0, msg.c_str());

  std::vector<Command> commands2{
                                Command{"AT+CMGR=1", "OK"},
                                Command{"AT+CMGR=1", "OK"},
      Command{"AT+CMGL=\"ALL\"", "OK"},
                                };

  for (const auto &c : commands2) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }

}

static char event_str[128];

static const char *gpio_irq_str[] = {
    "LEVEL_LOW",  // 0x1
    "LEVEL_HIGH", // 0x2
    "EDGE_FALL",  // 0x4
    "EDGE_RISE"   // 0x8
};

void gpio_event_string(char *buf, uint32_t events) {
  for (uint i = 0; i < 4; i++) {
    uint mask = (1 << i);
    if (events & mask) {
      // Copy this event string into the user string
      const char *event_str = gpio_irq_str[i];
      while (*event_str != '\0') {
        *buf++ = *event_str++;
      }
      events &= ~mask;

             // If more events add ", "
      if (events) {
        *buf++ = ',';
        *buf++ = ' ';
      }
    }
  }
  *buf++ = '\0';
}

void gpio_callback(uint gpio, uint32_t events) {
  // Put the GPIO event(s) that just happened into event_str
  // so we can print it
  gpio_event_string(event_str, events);
  printf("GPIO %d %s\n", gpio, event_str);
}

static const uint CHARGE_GPIO = 24;

void detectCharge() {
  gpio_init(CHARGE_GPIO);
  bool charging = gpio_get(CHARGE_GPIO) != 0;
  printf("CHARGING??? %s\n", charging ? "YES" : "NO");
  //gpio_set_irq_enabled_with_callback(CHARGE_GPIO, GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL, true, &gpio_callback);
  //while (true) {}
}

static const uint CALL_GPIO = 5;

// Unsolicited e.g. new sms  +CMTI

// UART1_RI Behaviors https://docs.rs-online.com/2190/0900766b815ca94b.pdf
// standby high
// sms 120ms low
// call low until hangup/establish
// UART1_RI is connected to Sim868 Pico Pin 7 (GP5) - Raspberry Pico Pin 7 (GP5) UART1 RX
void detectSMSorCall() {
  std::vector<Command> commands{
      Command{"AT+CMEE=1", "OK"},
      Command{"AT+CFUN?", "OK"},
                                Command{"AT+CSCLK?", "OK"},
                                Command{"AT+CLIP=1", "OK"},
      Command{"AT+CCALR=?", "OK"},
                                };

  for (const auto &c : commands) {
    const auto resp = Sim868::sendCommand(c.command);
    if (resp.find(c.response) == std::string::npos) {
      printf("command failed %s\n", c.command.c_str());
    }
  }

  gpio_init(CALL_GPIO);
  gpio_set_pulls(CALL_GPIO, true, false);
  //gpio_set_dir(CALL_GPIO, GPIO_IN);
  //gpio_set_irq_enabled_with_callback(CALL_GPIO, GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL, true, &gpio_callback);
  int i = 0;
  while (true) {
    sleep_ms(50);
    bool callOrSMS = gpio_get(CALL_GPIO) == 0;
    printf("call or sms? %d %s\n", i, callOrSMS ? "YES" : "NO");
    ++i;
    while (uart_is_readable_within_us(uart0, 2000)) {
      printf("%c", uart_getc(uart0));
    }
  }
}

int main(int argc, char *argv[]) {
  stdio_init_all();

  Sim868::init();
  Sim868::ledBlink();

  Sim868::start();

   checkNetwork();
   getBattery();
  // gsmLocation();
  // gpsLocation();
  // startCall();
  // sendSMS();
   checkSMS();
   detectCharge();
   detectSMSorCall();

  printf("\nBYEBYE\n");
  stdio_flush();

  return 0;
}
