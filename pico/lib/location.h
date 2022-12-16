#ifndef TATA_LIB_LOCATION
#define TATA_LIB_LOCATION

#include <cmath>
#include <concepts>
#include <cstdint>
#include <numbers>
#include <optional>

// TODO: chrono?

struct Location {
  double latitude{};
  double longitude{};
  float accuracy{};
  int64_t timestamp{};
};

template <typename T>
concept CLocator = requires(T o) {
  { o.getLocation() } -> std::same_as<std::optional<Location>>;
};

// https://stackoverflow.com/questions/27928/calculate-distance-between-two-latitude-longitude-points-haversine-formula
double getDistanceInMeter(double lat1, double lon1, double lat2, double lon2) {
  constexpr double earthRadiusInMeter{6371000};
  auto deg2rad = [](double deg) -> double {
    return deg * (std::numbers::pi / 180.0);
  };

  const auto dLat = deg2rad(lat2 - lat1);
  const auto dLon = deg2rad(lon2 - lon1);

  const auto a = std::pow(std::sin(dLat / 2.0), 2) +
                 std::pow(std::sin(dLon / 2.0), 2) * std::cos(deg2rad(lat1)) *
                     std::cos(deg2rad(lat2));
  const auto c = 2.0 * std::asin(std::sqrt(a));
  return earthRadiusInMeter * c;
}

#endif // TATA_LIB_LOCATION
