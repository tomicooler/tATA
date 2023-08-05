#ifndef TATA_LIB_TESTUTILS
#define TATA_LIB_TESTUTILS

#include <cassert>
#include <cmath>
#include <string>
#include <vector>

#include "executor.h"

struct LogTest {
  LogTest(std::string ctx) : ctx(std::move(ctx)) {
    printf("[%s]\n", this->ctx.c_str());
  }
  ~LogTest() { printf("[%s]  (PASS)\n", this->ctx.c_str()); }

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
    printf("%s", c.c_str());
    assert(idxExecute < expectedExecutes.size());
    assert(expectedExecutes.size() == returns.size());
    printf("  %s\n", expectedExecutes[idxExecute].c_str());
    assert(c == expectedExecutes[idxExecute]);
    auto resp = returns[idxExecute++];
    printf("%s\n", resp.c_str());
    return resp;
  };

  void sleep(std::chrono::nanoseconds d) {
    printf("  sleep: %ull\n",
           std::chrono::duration_cast<std::chrono::milliseconds>(d).count());
    assert(idxSleep < expectedSleeps.size());
    assert(d == expectedSleeps[idxSleep++]);
  }

  void write(const std::string &d) {
    printf("  write: %s\n", d.c_str());
    assert(idxWrite < expectedWrites.size());
    assert(d == expectedWrites[idxWrite++]);
  }

  void reboot() {
    printf("  reboot: %d\n", rebootCount);
    ++rebootCount;
    assert(rebootCount <= expectedRebootCount);
  }
};

static_assert(CExecutor<MockExecutor>);

bool equal(double a, double b, double epsilon = 0.1) {
  return std::fabs(a - b) < epsilon;
}

#endif // TATA_LIB_TESTUTILS
