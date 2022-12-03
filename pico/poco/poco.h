#ifndef TATA_POCO_POCO
#define TATA_POCO_POCO

#include <concepts>
#include <string>

#include "protector.h"
#include "watcher.h"

namespace PoCo {

template <typename T, typename U>
concept CSeriaziable = requires(T s, U o) {
  { s.serialize(o) } -> std::same_as<std::string>;
  { s.deserialize(o, std::string{}) } -> std::same_as<bool>;
};

template <typename T>
concept CSProtector = CSeriaziable<T, Protector>;

template <typename T>
concept CSWatcher = CSeriaziable<T, Watcher>;

} // namespace PoCo

#endif // TATA_POCO_POCO
