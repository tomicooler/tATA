#ifndef TATA_LIB_TESTUTILS
#define TATA_LIB_TESTUTILS

#include <cassert>
#include <fmt/chrono.h>
#include <fmt/color.h>
#include <fmt/printf.h>
#include <string>
#include <vector>

#include "executor.h"

struct LogTest {
  LogTest(std::string ctx) : ctx(std::move(ctx)) {
    fmt::print("[{}]\n", this->ctx);
  }
  ~LogTest() { fmt::print("[{}]  (PASS)\n", this->ctx); }

private:
  std::string ctx;
};

struct MockExecutor {
  std::vector<std::string> expectedExecutes{};
  std::vector<std::string> returns{};
  std::vector<std::chrono::nanoseconds> expectedSleeps{};
  std::vector<std::string> expectedWrites{};
  size_t expectedRebootCount{};

  // TODO: use unit test lib
  //       current tests do not ensure that expecteds are exhausted
  size_t idxExecute{};
  size_t idxSleep{};
  size_t idxWrite{};
  size_t rebootCount{};

  std::string execute(const std::string &c) {
    fmt::print(fg(fmt::color::yellow), "{}", c);
    assert(idxExecute < expectedExecutes.size());
    assert(expectedExecutes.size() == returns.size());
    fmt::print("  {}\n", expectedExecutes[idxExecute]);
    assert(c == expectedExecutes[idxExecute]);
    auto resp = returns[idxExecute++];
    fmt::print(fg(fmt::color::cyan), "{}\n", resp);
    return resp;
  };

  void sleep(std::chrono::nanoseconds d) {
    fmt::print("  sleep: {}\n", d);
    assert(idxSleep < expectedSleeps.size());
    assert(d == expectedSleeps[idxSleep++]);
  }

  void write(const std::string &d) {
    fmt::print("  write: {}\n", d);
    assert(idxWrite < expectedWrites.size());
    assert(d == expectedWrites[idxWrite++]);
  }

  void reboot() {
    fmt::print("  reboot: {}\n", rebootCount);
    ++rebootCount;
    assert(rebootCount <= expectedRebootCount);
  }
};

static_assert(CExecutor<MockExecutor>);

bool equal(double a, double b, double epsilon = 0.1) {
  return std::fabs(a - b) < epsilon;
}

#endif // TATA_LIB_TESTUTILS
