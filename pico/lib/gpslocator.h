#ifndef TATA_LIB_GPS_LOCATION
#define TATA_LIB_GPS_LOCATION

#include <chrono>
#include <iomanip>
#include <vector>

#include "executor.h"
#include "location.h"
#include "scope_exit.h"
#include "utils.h"

namespace AT {
constexpr char gpsPowerON[] = "AT+CGNSPWR=1";
constexpr char gpsPowerOFF[] = "AT+CGNSPWR=0";
constexpr char gpsInfo[] = "AT+CGNSINF";
} // namespace AT

namespace GPS {

// SIM868_Series_GNSS_Application_Note_V1.02.pdf
enum class Csv {
  RunStatus,
  FixStatus,
  UTCDateTime,
  Latitude,
  Longitude,
  MSLAltitude,
  SpeedOverGround,
  CourseOverGround,
  FixMode,
  Reserved1,
  HDOP,
  PDOP,
  VDOP,
  Reserved2,
  GNSSSatelitesInView,
  GNSSSatelitesUsed,
  GLONASSSatelitesUsed,
  Reserved3,
  CNOMax,
  HorizontalPositionAccuracy,
  VPA,
};

template <CExecutor Executor> struct Locator {
  static constexpr int MaxRetries = 30;

  constexpr Locator(Executor e) : ex(e) {}

  [[nodiscard]] std::optional<Location> getLocation() {
    if (!isSuccessfulReturn(ex.execute(AT::gpsPowerON))) {
      return std::nullopt;
    }
    const auto gpsOFF = scope_exit{
        [&]() { [[maybe_unused]] const auto _ = ex.execute(AT::gpsPowerOFF); }};

    std::optional<Location> loc;
    for (int i = 0; i < MaxRetries; ++i) {
      if (const auto ret = ex.execute(AT::gpsInfo); isSuccessfulReturn(ret)) {
        constexpr auto size = static_cast<size_t>(Csv::VPA) + 1;
        std::vector<std::string> tokens;
        tokens.reserve(size);
        Utils::tokenize(tokens, ret, ",");
        if (tokens.size() == size) {
          const auto parseDouble = [&](Csv at) -> double {
            return strtod(tokens.at(static_cast<size_t>(at)).c_str(), nullptr);
          };

          const auto parseTimestamp = [&]() -> int64_t {
            const auto datetimeutc =
                tokens.at(static_cast<size_t>(Csv::UTCDateTime));
            std::tm tm = {};
            std::stringstream ss{datetimeutc};
            ss >> std::get_time(&tm, "%Y%m%d%H%M%S.000"); // yyyyMMddhhmmss.sss
            auto tp = std::chrono::system_clock::from_time_t(std::mktime(&tm));
            return std::chrono::time_point_cast<std::chrono::seconds>(tp)
                .time_since_epoch()
                .count();
          };

          loc.emplace(Location{
              parseDouble(Csv::Latitude), parseDouble(Csv::Longitude),
              static_cast<float>(parseDouble(Csv::HorizontalPositionAccuracy)),
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

} // namespace GPS

#endif // TATA_LIB_GPS_LOCATION
