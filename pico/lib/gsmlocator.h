#ifndef TATA_LIB_GSM_LOCATION
#define TATA_LIB_GSM_LOCATION

#include <chrono>
#include <iomanip>
#include <vector>

#include "executor.h"
#include "location.h"
#include "scope_exit.h"
#include "utils.h"

namespace AT {
constexpr char gprsON[] = "AT+CGATT=1";
constexpr char gprsOFF[] = "AT+CGATT=0";
constexpr char setAPN[] = "AT+CSTT=\"online\""; // TODO configurable APN
constexpr char bringUpWirelessGPRSorCSD[] = "AT+CIICR";
constexpr char getLocalIPAddress[] = "AT+CIFSR";
constexpr char setBearerGPRS[] = "AT+SAPBR=3,1,\"Contype\",\"GPRS\"";
constexpr char setBearerAPN[] =
    "AT+SAPBR=3,1,\"APN\",\"online\""; // TODO configurable APN
constexpr char activateBearer[] = "AT+SAPBR=1,1";
constexpr char deactivateBearer[] = "AT+SAPBR=0,1";
constexpr char setLBSAddressFree[] = "AT+CLBSCFG=1,3,\"lbs-simcom.com:3002\"";
constexpr char getLatLong[] = "AT+CLBS=4,1";
} // namespace AT

namespace GSM {

// SIM800_Series_GSM_Location_Application_Note_V1.03.pdf
enum class Csv {
  LocationCode,
  Longitude,
  Latitude,
  Accuracy,
  Date,
  Time,
};

enum class LocationType {
  WGS84,
  GCJ02, // no plan to launch my car to Mars or China
};

template <CExecutor Executor> struct Locator {
  static constexpr int MaxRetries = 30;

  constexpr Locator(Executor e) : ex(e) {}

  [[nodiscard]] std::optional<Location> getLocation() {
    if (!isSuccessfulReturn(ex.execute(AT::gprsON))) {
      return std::nullopt;
    }
    const auto gprsOFF = scope_exit{
        [&]() { [[maybe_unused]] const auto _ = ex.execute(AT::gprsOFF); }};

    if (!isSuccessfulReturn(ex.execute(AT::setAPN))) {
      return std::nullopt;
    }

    if (!isSuccessfulReturn(ex.execute(AT::bringUpWirelessGPRSorCSD))) {
      return std::nullopt;
    }

    if (!isSuccessfulReturn(ex.execute(AT::getLocalIPAddress))) {
      return std::nullopt;
    }

    if (!isSuccessfulReturn(ex.execute(AT::setBearerAPN))) {
      return std::nullopt;
    }

    if (!isSuccessfulReturn(ex.execute(AT::setBearerGPRS))) {
      return std::nullopt;
    }

    if (!isSuccessfulReturn(ex.execute(AT::activateBearer))) {
      return std::nullopt;
    }
    const auto deactivateBearer = scope_exit{[&]() {
      [[maybe_unused]] const auto _ = ex.execute(AT::deactivateBearer);
    }};

    if (!isSuccessfulReturn(ex.execute(AT::setLBSAddressFree))) {
      return std::nullopt;
    }

    std::optional<Location> loc;
    for (int i = 0; i < MaxRetries; ++i) {
      if (const auto ret = ex.execute(AT::getLatLong);
          isSuccessfulReturn(ret)) {
        constexpr auto size = static_cast<size_t>(Csv::Time) + 1;
        std::vector<std::string> tokens;
        tokens.reserve(size);
        Utils::tokenize(tokens, ret, ",");
        if (tokens.size() == size) {
          const auto parseDouble = [&](Csv at) -> double {
            return strtod(tokens.at(static_cast<size_t>(at)).c_str(), nullptr);
          };
          const auto parseTimestamp = [&]() -> int64_t {
            const auto date = tokens.at(static_cast<size_t>(Csv::Date));
            const auto time = tokens.at(static_cast<size_t>(Csv::Time));
            std::tm tm = {};
            std::stringstream ss{date + " " + time};
            ss >> std::get_time(&tm, "%d/%m/%y %H:%M:%S");
            auto tp = std::chrono::system_clock::from_time_t(std::mktime(&tm));
            return std::chrono::time_point_cast<std::chrono::seconds>(tp)
                .time_since_epoch()
                .count();
          };

          loc.emplace(Location{parseDouble(Csv::Latitude),
                               parseDouble(Csv::Longitude),
                               static_cast<float>(parseDouble(Csv::Accuracy)),
                               parseTimestamp()});
          break;
        }
      }
    }

    return loc;
  }

private:
  Executor ex;
};

static_assert(CLocator<Locator<NoopExecutor>>);

} // namespace GSM

#endif // TATA_LIB_GSM_LOCATION
