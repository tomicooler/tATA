#ifndef TATA_POCO_DEBUG
#define TATA_POCO_DEBUG

#include <chrono>
#include <iomanip>
#include <sstream>
#include <string>

#include "protector.h"
#include "watcher.h"

// TODO: checkout the fmt library
//       mostly for debugging, but human serialization utilises this too

namespace PoCo {

template <typename T>
concept BoolObject = requires(T a) {
  a.value = true;
};

template <typename T>
static inline std::string to_string_with_precision(const T o, const int n = 6) {
  std::ostringstream out;
  out.precision(n);
  out << std::fixed << o;
  return out.str();
}

static inline std::string to_string(float o) {
  return to_string_with_precision(o, 2);
}

static inline std::string to_string(double o) {
  return to_string_with_precision(o, 7);
}

static inline std::string unix_time_to_string(int64_t o) {
  auto seconds_since_epoch = std::chrono::system_clock::to_time_t(
      std::chrono::time_point<std::chrono::system_clock>{
          std::chrono::milliseconds{o}});
  std::ostringstream out;
  out << std::put_time(std::localtime(&seconds_since_epoch), "%D %T %Z");
  return out.str();
}

static inline std::string to_string(const Position &o) {
  using namespace std::string_literals;
  return to_string(o.latitude) + ","s + to_string(o.longitude);
}

static inline std::string to_string(const CarLocation &o) {
  using namespace std::string_literals;
  return "["s + to_string(o.position) + " " + to_string(o.accuracy) + "m "s +
         to_string(o.battery * 100.0f) + "% "s +
         unix_time_to_string(o.timestamp) + "]"s;
}

static inline std::string to_string(const ParkLocation &o) {
  using namespace std::string_literals;
  return "["s + to_string(o.position) + " " + to_string(o.accuracy) + "m]"s;
}

static inline std::string to_string(const Status &o) {
  switch (o.type) {
  case Status::Type::ParkingDetected:
    return "ParkingDetected";
  case Status::Type::ParkingUpdated:
    return "ParkingUpdated";
  case Status::Type::CarTheftDetected:
    return "CarTheftDetected";
  }
  return {};
}

static inline std::string to_string(const BoolObject auto &o) {
  return o.value ? "true" : "false";
}

static inline std::string to_string(const ReceiverInfo &o) {
  using namespace std::string_literals;
  return "["s + [o]() -> std::string {
    switch (o.type) {
    case ReceiverInfo::Type::Gcm:
      return "GCM";
    case ReceiverInfo::Type::SmsHuman:
      return "Human";
    case ReceiverInfo::Type::SmsMachine:
      return "Machine";
    case ReceiverInfo::Type::Service:
      return "Service";
    }
    return {};
  }() + " "s + o.phoneNumber +
                             "]"s;
}

template <typename T>
static inline std::string to_string(const std::optional<T> &o) {
  return o.has_value() ? to_string(*o) : "null";
}

static inline std::string to_string(const Protector &o) {
  using namespace std::string_literals;
  return "{"s + to_string(o.carLocation) + " "s + to_string(o.parkLocation) +
         " "s + to_string(o.status) + " "s + to_string(o.service) + "}"s;
}

static inline std::string to_string(const Watcher &o) {
  using namespace std::string_literals;
  return "{"s + to_string(o.call) + " "s + to_string(o.refresh) + " "s +
         to_string(o.park) + " "s + to_string(o.receiver) + " "s +
         to_string(o.service) + "}"s;
}

} // namespace PoCo

#endif // TATA_POCO_DEBUG
