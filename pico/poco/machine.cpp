#include "machine.h"
#include "parizer.h"

using namespace PoCo;

// TODO: more generelization if possible, static reflection would be awesome
// https://youtu.be/myhB8ZlwOlE?t=530

std::string Machine::ProtectorS::serialize(const Protector &protector) const {
  std::string ret;
  ret += details::serialize(protector.carLocation) +
         std::string{details::DELIMITER};
  ret += details::serialize(protector.parkLocation) +
         std::string{details::DELIMITER};
  ret += details::serialize(protector.status) + std::string{details::DELIMITER};
  ret += details::serialize(protector.service);
  return ret;
}

bool Machine::ProtectorS::deserialize(Protector &protector,
                                      const std::string &str) const {
  auto tokens = details::tokenize(str);

  if (!details::deserialize(protector.carLocation, tokens))
    return false;
  if (!details::deserialize(protector.parkLocation, tokens))
    return false;
  if (!details::deserialize(protector.status, tokens))
    return false;
  if (!details::deserialize(protector.service, tokens))
    return false;

  return tokens.empty();
}

std::string Machine::WatcherS::serialize(const Watcher &watcher) const {
  std::string ret;
  ret += details::serialize(watcher.call) + std::string{details::DELIMITER};
  ret += details::serialize(watcher.refresh) + std::string{details::DELIMITER};
  ret += details::serialize(watcher.park) + std::string{details::DELIMITER};
  ret += details::serialize(watcher.receiver) + std::string{details::DELIMITER};
  ret += details::serialize(watcher.service);
  return ret;
}

bool Machine::WatcherS::deserialize(Watcher &watcher,
                                    const std::string &str) const {
  auto tokens = details::tokenize(str);

  if (!details::deserialize(watcher.call, tokens))
    return false;
  if (!details::deserialize(watcher.refresh, tokens))
    return false;
  if (!details::deserialize(watcher.park, tokens))
    return false;
  if (!details::deserialize(watcher.receiver, tokens))
    return false;
  if (!details::deserialize(watcher.service, tokens))
    return false;

  return tokens.empty();
}
