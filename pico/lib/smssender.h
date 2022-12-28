#ifndef TATA_LIB_SMSSENDER
#define TATA_LIB_SMSSENDER

#include <chrono>
#include <fmt/printf.h>

#include "executor.h"

namespace AT {
constexpr char setSMSTextMode[] = "AT+CMGF=1";
constexpr char sendSMS[] = "AT+CMGS=\"{}\"";
} // namespace AT

template <CExecutor Executor> struct SMSSEnder {
  constexpr SMSSEnder(Executor e) : ex(e) {}

  void send(std::string_view number, std::string text) {
    if (!isSuccessfulReturn(ex.execute(AT::setSMSTextMode))) {
      return;
    }

    if (const auto ret = ex.execute(fmt::format(AT::sendSMS, number));
        ret != ">") {
      return;
    }

    ex.write(text.append(std::string{'\x1A'}));
  }

private:
  Executor ex;
};

#endif // TATA_LIB_SMSSENDER
