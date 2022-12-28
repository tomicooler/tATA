#ifndef TATA_LIB_BATTERY
#define TATA_LIB_BATTERY

#include <chrono>
#include <fmt/printf.h>
#include <iomanip>
#include <vector>

#include "executor.h"
#include "utils.h"

namespace AT {
constexpr char checkBattery[] = "AT+CBC";
} // namespace AT

namespace Bat {

// SIM800_Series_AT_Command_Manual_V1.11.pdf
enum class Csv {
  ChargingSatus,
  Percentage,
  Voltage,
};

struct Battery {
  enum class Status { NotCharging, Charging, Full };

  Status status{Status::NotCharging};
  float percentage{};
  int milliVolts{};
};

template <CExecutor Executor> struct BatteryChecker {
  constexpr BatteryChecker(Executor e) : ex(e) {}

  [[nodiscard]] Battery getBattery() {
    if (auto ret = ex.execute(AT::checkBattery); isSuccessfulReturn(ret)) {
      constexpr std::string_view toErase = "+CBC: ";
      const size_t pos = ret.find(toErase);
      if (pos != std::string::npos) {
        ret.erase(pos, toErase.length());
      }

      constexpr auto size = static_cast<size_t>(Csv::Voltage) + 1;
      std::vector<std::string> tokens;
      tokens.reserve(size);
      Utils::tokenize(tokens, ret, ",");
      if (tokens.size() == size) {
        const auto parseInt = [&](Csv at) -> int {
          return std::stoi(tokens.at(static_cast<size_t>(at)).c_str(), nullptr,
                           10);
        };

        const auto parseStatus = [&]() -> Battery::Status {
          const auto s = parseInt(Csv::ChargingSatus);
          switch (s) {
          case 0:
            return Battery::Status::NotCharging;
          case 1:
            return Battery::Status::Charging;
          case 2:
            return Battery::Status::Full;
          }

          return Battery::Status::NotCharging;
        };

        return Battery{parseStatus(),
                       std::clamp(static_cast<float>(parseInt(Csv::Percentage)),
                                  0.f, 100.f) /
                           100.0f,
                       parseInt(Csv::Voltage)};
      }
    }
    return Battery{};
  }

private:
  Executor ex;
};
} // namespace Bat

#endif // TATA_LIB_BATTERY
