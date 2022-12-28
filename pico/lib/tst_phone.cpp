#include "caller.h"
#include "network.h"
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

void testNetwork() {
  Network n{MockExecutor{.expectedExecutes =
                             {
                                 {AT::enableEcho},
                                 {AT::init},
                                 {AT::enableEcho},
                                 {AT::init},
                                 {AT::checkNetworkStatus},
                                 {AT::checkNetworkStatus},
                                 {AT::checkPIN},
                                 {AT::reportSignalQuality},
                                 {AT::checkNetworkOperator},
                             },
                         .returns =
                             {
                                 {AT::OK},
                                 {"FAIL"},
                                 {AT::OK},
                                 {AT::OK},
                                 {"0,2\r\n\r\nOK\r\n"},
                                 {"0,1\r\n\r\nOK\r\n"},
                                 {"+CPIN: READY\r\n\r\nOK\r\n"},
                                 {"+CSQ: 19,0\r\n\r\nOK\r\n"},
                                 {"+COPS: 0,0,\"PANNON GSM\"\r\n\r\nOK\r\n"},
                             },
                         .expectedSleeps = {2s, 2s, 2s, 2s},
                         .expectedRebootCount = 2}};

  assert(n.start() == true);
}

int main(int argc, char *argv[]) {
  testCall();
  testSMS();
  testNetwork();
  return 0;
}
