#include "human.h"
#include "../lib/location.h"
#include "debug.h"

using namespace PoCo;

std::string Human::ProtectorS::serialize(const Protector &protector) const {
  std::ostringstream out;

  if (protector.carLocation.has_value()) {
    const auto &l = *protector.carLocation;
    out << "http://maps.google.com/?q=" << PoCo::to_string(l.position) << "\n\n"
        << PoCo::to_string(l.accuracy) << " meters, "
        << PoCo::to_string(l.battery * 100.0f) << " %, "
        << PoCo::unix_time_to_string(l.timestamp) << "\n\n";
  }

  if (protector.service.has_value() && protector.service->value) {
    out << "Service on\n\n";
  }

  if (!protector.carLocation.has_value() &&
      protector.parkLocation.has_value()) {
    // If there is no car location available, but we have the last park location
    // send at least that park location
    const auto &l = *protector.parkLocation;
    out << "Last park location\n\nhttp://maps.google.com/?q="
        << PoCo::to_string(l.position) << "\n\n"
        << PoCo::to_string(l.accuracy) << " meters\n\n";
  } else if (protector.parkLocation.has_value()) {
    out << "Park distance "
        << PoCo::to_string(static_cast<float>(
               getDistanceInMeter(protector.carLocation->position.latitude,
                                  protector.carLocation->position.longitude,
                                  protector.parkLocation->position.latitude,
                                  protector.parkLocation->position.longitude)))
        << " meters\n\n";
  }

  return out.str();
}

bool Human::ProtectorS::deserialize(Protector &protector,
                                    const std::string &str) const {
  return {};
}

std::string Human::WatcherS::serialize(const Watcher &watcher) const {
  return {};
}

bool Human::WatcherS::deserialize(Watcher &watcher,
                                  const std::string &str) const {
  if (str == "location") {
    watcher.refresh.emplace(Refresh{true});
  } else if (str == "call") {
    watcher.call.emplace(Call{true});
  } else if (str == "park on") {
    watcher.park.emplace(Park{true});
  } else if (str == "park off") {
    watcher.park.emplace(Park{false});
  } else if (str == "service on") {
    watcher.service.emplace(Service{true});
  } else if (str == "service off") {
    watcher.service.emplace(Service{false});
  } else {
    return false;
  }

  return true;
}
