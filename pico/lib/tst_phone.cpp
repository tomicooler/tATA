#include "caller.h"
#include "smssender.h"
#include "testutils.h"

using namespace std::chrono_literals;

constexpr std::string_view number = "+36301112233";

void testCall() {
  LogTest t{"testCall"};
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

void testSMS() {
  LogTest t{"testSMS"};
  SMSSEnder s{
      MockExecutor{.expectedExecutes = {{AT::setSMSTextMode},
                                        fmt::format(AT::sendSMS, number)},
                   .returns =
                       {
                           {AT::OK},
                           {">"},
                       },
                   .expectedWrites = {"hello world!\x1A"}}};
  s.send(number, "hello world!");
}

int main(int argc, char *argv[]) {
  testCall();
  testSMS();
  return 0;
}
