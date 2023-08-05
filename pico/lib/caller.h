#ifndef TATA_LIB_CALLER
#define TATA_LIB_CALLER

#include <chrono>

#include "executor.h"
#include "utils.h"

namespace AT {
constexpr char setAUXAudio[] = "AT+CHFA=1";
constexpr char callNumber[] = "ATD{};";
constexpr char hangup[] = "AT+CHUP;";
} // namespace AT

template <CExecutor Executor> struct Caller {
  constexpr Caller(Executor e) : ex(e) {}

  void call(std::string_view number, std::chrono::nanoseconds duration) {
    if (!isSuccessfulReturn(ex.execute(AT::setAUXAudio))) {
      return;
    }

    if (!isSuccessfulReturn(ex.execute(fmt::format(AT::callNumber, number)))) {
      return;
    }

    ex.sleep(duration);

    if (!isSuccessfulReturn(ex.execute(AT::hangup))) {
      return;
    }
  }

private:
  Executor ex;
};

#endif // TATA_LIB_CALLER
