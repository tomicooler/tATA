#ifndef TATA_POCO_MACHINE
#define TATA_POCO_MACHINE

#include "poco.h"

namespace PoCo {

namespace Machine {

struct ProtectorS {
  [[nodiscard]] std::string serialize(const Protector &protector) const;
  [[nodiscard]] bool deserialize(Protector &protector,
                                 const std::string &str) const;
};

static_assert(CSProtector<ProtectorS>);

struct WatcherS {
  [[nodiscard]] std::string serialize(const Watcher &watcher) const;
  [[nodiscard]] bool deserialize(Watcher &watcher,
                                 const std::string &str) const;
};

static_assert(CSWatcher<WatcherS>);

} // namespace Machine

} // namespace PoCo

#endif // TATA_POCO_MACHINE
