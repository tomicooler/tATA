#ifndef TATA_POCO_HUMAN
#define TATA_POCO_HUMAN

#include "poco.h"

namespace PoCo {

namespace Human {

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

} // namespace Human

} // namespace PoCo

#endif // TATA_POCO_HUMAN
