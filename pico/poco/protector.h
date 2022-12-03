#ifndef TATA_POCO_PROTECTOR
#define TATA_POCO_PROTECTOR

#include <cstdint>
#include <optional>

#include "common.h"

namespace PoCo {

struct Position {
  double latitude{};
  double longitude{};
};

struct CarLocation {
  Position position{};
  float accuracy{};
  float battery{};
  int64_t timestamp{};
};

struct ParkLocation {
  Position position{};
  float accuracy{};
};

struct Status {
  enum class Type : uint8_t {
    ParkingDetected,
    ParkingUpdated,
    CarTheftDetected
  };

  Type type;
};

struct Protector {
  std::optional<CarLocation> carLocation;
  std::optional<ParkLocation> parkLocation;
  std::optional<Status> status;
  std::optional<Service> service;
};

} // namespace PoCo

#endif // TATA_POCO_PROTECTOR
