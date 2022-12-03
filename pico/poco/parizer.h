#ifndef TATA_POCO_PARIZER
#define TATA_POCO_PARIZER

#include <algorithm>
#include <concepts>
#include <list>

#include "debug.h"
#include "protector.h"
#include "watcher.h"

namespace PoCo {

namespace details {
constexpr std::string_view DELIMITER = " ";
constexpr std::string_view SPACE = "_";
constexpr std::string_view NULLS = "*";
constexpr std::string_view TRUE = "t";
constexpr std::string_view FALSE = "f";
constexpr std::string_view EMPTY = ";";
constexpr int64_t FLOAT_PRECISION = 1000000LL;
constexpr int64_t DOUBLE_PRECISION = 10000000000LL;

constexpr char DIGITS[] = "0123456789abcdefghijklmnopqrstuvwxyz";
constexpr int BASE = std::size(DIGITS) - 1;

std::string numToString(std::integral auto value, int base = BASE) {
  if (base < 2 || base > BASE) {
    return {};
  }

  std::string buf;
  auto quotient = value;
  do {
    buf += DIGITS[std::abs(quotient % base)];
    quotient /= base;
  } while (quotient);

  if (value < 0) {
    buf += "-";
  }

  std::reverse(std::begin(buf), std::end(buf));
  return buf;
}

int64_t numFromString(const std::string &str, int base = BASE) {
  return strtoll(str.c_str(), nullptr, base);
}

float toFloat(const std::string &str) {
  return static_cast<float>(numFromString(str)) /
         static_cast<float>(FLOAT_PRECISION);
}

double toDouble(const std::string &str) {
  return static_cast<double>(numFromString(str)) /
         static_cast<double>(DOUBLE_PRECISION);
}

void replaceAll(std::string &str, const std::string_view &from,
                const std::string_view &to) {
  if (from.empty())
    return;
  size_t start_pos = 0;
  while ((start_pos = str.find(from, start_pos)) != std::string::npos) {
    str.replace(start_pos, from.length(), to);
    start_pos += to.length();
  }
}

std::string escape(std::string str) {
  if (str.empty())
    return std::string{EMPTY};

  replaceAll(str, SPACE, std::string{SPACE} + std::string{SPACE});
  replaceAll(str, std::string{DELIMITER} + std::string{DELIMITER},
             std::string{SPACE} + std::string{SPACE} + std::string{SPACE} +
                 std::string{SPACE});
  replaceAll(str, DELIMITER, SPACE);
  replaceAll(str, NULLS, std::string{NULLS} + std::string{NULLS});
  replaceAll(str, EMPTY, std::string{EMPTY} + std::string{EMPTY});
  return str;
}

std::string unescape(std::string str) {
  if (str == EMPTY)
    return {};

  replaceAll(str, std::string{SPACE} + std::string{SPACE}, SPACE);
  replaceAll(str, SPACE, DELIMITER);
  replaceAll(str, std::string{NULLS} + std::string{NULLS}, NULLS);
  replaceAll(str, std::string{EMPTY} + std::string{EMPTY}, EMPTY);
  return str;
}

using Tokens = std::list<std::string>;

Tokens tokenize(const std::string &str) {
  Tokens ret;

  size_t start{};
  size_t end{};

  while ((start = str.find_first_not_of(DELIMITER, end)) != std::string::npos) {
    end = str.find(DELIMITER, start);
    ret.push_back(str.substr(start, end - start));
  }

  return ret;
}

std::string serialize(float value) {
  return numToString(static_cast<int64_t>(value * FLOAT_PRECISION));
}

std::string serialize(double value) {
  return numToString(static_cast<int64_t>(value * DOUBLE_PRECISION));
}

std::string serialize(std::integral auto value) { return numToString(value); }

std::string serialize(const std::string &str) { return details::escape(str); }

std::string serialize(const Position &position) {
  std::string ret;
  ret += serialize(position.latitude) + std::string{details::DELIMITER};
  ret += serialize(position.longitude);
  return ret;
}

std::string serialize(const CarLocation &carLocation) {
  std::string ret;
  ret += serialize(carLocation.position) + std::string{details::DELIMITER};
  ret += serialize(carLocation.accuracy) + std::string{details::DELIMITER};
  ret += serialize(carLocation.battery) + std::string{details::DELIMITER};
  ret += serialize(carLocation.timestamp);
  return ret;
}

std::string serialize(const ParkLocation &parkLocation) {
  std::string ret;
  ret += serialize(parkLocation.position) + std::string{details::DELIMITER};
  ret += serialize(parkLocation.accuracy);
  return ret;
}

std::string serialize(const Status &status) {
  return serialize(static_cast<int64_t>(status.type));
}

std::string serialize(const BoolObject auto &flag) {
  return flag.value ? std::string{details::TRUE} : std::string{details::FALSE};
}

std::string serialize(const ReceiverInfo &receiver) {
  std::string ret;
  ret += serialize(static_cast<int64_t>(receiver.type)) +
         std::string{details::DELIMITER};
  ret += serialize(receiver.phoneNumber);
  return ret;
}

template <typename T> std::string serialize(const std::optional<T> &opt) {
  if (!opt.has_value())
    return std::string{details::NULLS};
  return serialize(*opt);
}

bool deserialize(BoolObject auto &flag, Tokens &tokens) {
  if (tokens.empty())
    return false;
  const auto str = tokens.front();
  tokens.pop_front();
  if (str == details::TRUE) {
    flag.value = true;
  } else if (str == details::FALSE) {
    flag.value = false;
  } else {
    return false;
  }
  return true;
}

bool deserialize(std::string &str, Tokens &tokens) {
  if (tokens.empty())
    return false;

  const auto tmp = tokens.front();
  tokens.pop_front();
  str = details::unescape(tmp);
  return true;
}

bool deserialize(float &value, Tokens &tokens) {
  if (tokens.empty())
    return false;

  const auto tmp = tokens.front();
  tokens.pop_front();
  value = details::toFloat(tmp);
  return true;
}

bool deserialize(double &value, Tokens &tokens) {
  if (tokens.empty())
    return false;

  const auto tmp = tokens.front();
  tokens.pop_front();
  value = details::toDouble(tmp);
  return true;
}

bool deserialize(int64_t &value, Tokens &tokens) {
  if (tokens.empty())
    return false;

  const auto tmp = tokens.front();
  tokens.pop_front();
  value = details::numFromString(tmp);
  return true;
}

bool deserialize(ReceiverInfo &receiverInfo, Tokens &tokens) {
  int64_t tmp{-1};
  if (!deserialize(tmp, tokens))
    return false;

  if (tmp >= static_cast<int64_t>(ReceiverInfo::Type::Gcm) &&
      tmp <= static_cast<int64_t>(ReceiverInfo::Type::Service)) {
    receiverInfo.type = static_cast<ReceiverInfo::Type>(tmp);
  } else {
    return false;
  }

  if (!deserialize(receiverInfo.phoneNumber, tokens))
    return false;

  return true;
}

bool deserialize(Position &position, Tokens &tokens) {
  if (!deserialize(position.latitude, tokens))
    return false;
  if (!deserialize(position.longitude, tokens))
    return false;
  return true;
}

bool deserialize(CarLocation &carLocation, Tokens &tokens) {
  if (!deserialize(carLocation.position, tokens))
    return false;
  if (!deserialize(carLocation.accuracy, tokens))
    return false;
  if (!deserialize(carLocation.battery, tokens))
    return false;
  if (!deserialize(carLocation.timestamp, tokens))
    return false;
  return true;
}

bool deserialize(ParkLocation &parkLocation, Tokens &tokens) {
  if (!deserialize(parkLocation.position, tokens))
    return false;
  if (!deserialize(parkLocation.accuracy, tokens))
    return false;
  return true;
}

bool deserialize(Status &status, Tokens &tokens) {
  int64_t tmp{-1};
  if (!deserialize(tmp, tokens))
    return false;

  if (tmp >= static_cast<int64_t>(Status::Type::ParkingDetected) &&
      tmp <= static_cast<int64_t>(Status::Type::CarTheftDetected)) {
    status.type = static_cast<Status::Type>(tmp);
    return true;
  }
  return false;
}

template <typename T> bool deserialize(std::optional<T> &opt, Tokens &tokens) {
  if (tokens.empty())
    return false;
  const auto str = tokens.front();
  if (str == details::NULLS) {
    tokens.pop_front();
    opt = std::nullopt;
  } else {
    opt.emplace(T{});
    return deserialize(*opt, tokens);
  }
  return true;
}

} // namespace details

} // namespace PoCo

#endif // TATA_POCO_PARIZER
