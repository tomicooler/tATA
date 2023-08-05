#ifndef TATA_LIB_NETWORK
#define TATA_LIB_NETWORK

#include <chrono>

#include "executor.h"

// NOTE: PIN must be disabled on the Sim card!

namespace AT {
constexpr char enableEcho[] = "ATE1";
constexpr char init[] = "AT";
constexpr char checkNetworkStatus[] = "AT+CGREG?";
constexpr char checkPIN[] = "AT+CPIN?";
constexpr char reportSignalQuality[] = "AT+CSQ";
constexpr char checkNetworkOperator[] = "AT+COPS?";
} // namespace AT

template <CExecutor Executor> struct Network {
  constexpr Network(Executor e) : ex(e) {}

  [[nodiscard]] bool start() {
    using namespace std::chrono_literals;

    printf("#Network - starting up\n");

    for (;;) {
      if (!isSuccessfulReturn(ex.execute(AT::enableEcho))) {
        ex.sleep(2s);
        continue;
      }
      ex.sleep(2s);

      if (!isSuccessfulReturn(ex.execute(AT::init))) {
        printf("#Network - could not init, let's reboot...\n");
        ex.reboot();
        ex.sleep(2s);
      } else {
        printf("#Network - init successful\n");
        break;
      }
    }

    printf("#Network - checking network status...\n");
    for (;;) {
      if (const auto ret = ex.execute(AT::checkNetworkStatus);
          !isSuccessfulReturn(ret) || ret.find("0,1") == std::string::npos) {
        ex.sleep(2s);
        continue;
      } else {
        break;
      }
    }

    if (!isSuccessfulReturn(ex.execute(AT::checkPIN))) {
      return false;
    }

    if (!isSuccessfulReturn(ex.execute(AT::reportSignalQuality))) {
      return false;
    }

    if (!isSuccessfulReturn(ex.execute(AT::checkNetworkOperator))) {
      return false;
    }

    printf("#Network - started\n");
    return true;
  }

private:
  Executor ex;
};

#endif // TATA_LIB_NETWORK
