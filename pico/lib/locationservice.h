#ifndef TATA_LIB_LOCATION_SERVICE
#define TATA_LIB_LOCATION_SERVICE

#include <tuple>
#include <vector>

#include "location.h"

template <typename T, typename... Ts>
concept same_as_any = (... or std::same_as<T, Ts>);

template <CLocator... CLocators> struct LocationService {
  template <same_as_any<CLocators...> T> auto addLocator(T locator) {
    return std::get<std::vector<T>>(locators).push_back(locator);
  }

  [[nodiscard]] std::optional<Location> getLocation() {
    std::optional<Location> loc;
    std::apply(
        [this, &loc](auto &...ll) {
          ((getFirstLoc(ll, loc) ? false : (getFirstLoc(ll, loc), true)) &&
           ...);
        },
        locators);
    return loc;
  };

private:
  using Locators = std::tuple<std::vector<CLocators>...>;

  bool getFirstLoc(auto &ll, std::optional<Location> &loc) {
    for (auto &locator : ll) {
      loc = locator.getLocation();
      if (loc.has_value()) {
        return true;
      }
    }

    return false;
  }

  Locators locators{};
};

#endif // TATA_LIB_LOCATION_SERVICE
