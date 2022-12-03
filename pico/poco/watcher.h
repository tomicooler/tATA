#ifndef TATA_POCO_WATCHER
#define TATA_POCO_WATCHER

#include <cstdint>
#include <optional>
#include <string>

#include "common.h"

namespace PoCo {

struct Call {
  bool value{};
};

struct Refresh {
  bool value{};
};

struct Park {
  bool value{};
};

struct ReceiverInfo {
  enum class Type : uint8_t { Gcm, SmsHuman, SmsMachine, Service };

  Type type;
  std::string phoneNumber;
};

struct Watcher {
  std::optional<Call> call;
  std::optional<Refresh> refresh;
  std::optional<Park> park;
  std::optional<ReceiverInfo> receiver;
  std::optional<Service> service;
};

} // namespace PoCo

#endif // TATA_POCO_WATCHER
