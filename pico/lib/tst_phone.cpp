#include "caller.h"
#include "testutils.h"

using namespace std::chrono_literals;

void testCall() {
  LogTest t{"testCall"};
  constexpr std::string_view number = "+36301112233";
  Caller c{
      MockExecutor{.expectedExecutes = {{AT::setAUXAudio},
                                        fmt::format(AT::callNumber, number),
                                        {AT::hangup}},
                   .returns =
                       {
                           {AT::OK},
                           {AT::OK},
                           {AT::OK},
                       },
                   .expectedSleeps = {100ms}}};
  c.call(number, 100ms);
}

int main(int argc, char *argv[]) {
  testCall();
  return 0;
}
