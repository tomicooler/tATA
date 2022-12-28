#include "battery.h"
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
  LogTest t{"testNetwork"};
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

void testBattery() {
  {
    LogTest t{"testBattery not charging"};
    Bat::BatteryChecker b{MockExecutor{
        .expectedExecutes =
            {
                {AT::checkBattery},
            },
        .returns =
            {
                {"+CBC: 0,50,300\r\n\r\nOK\r\n"},
            },
    }};
    const auto bat = b.getBattery();
    assert(bat.status == Bat::Battery::Status::NotCharging);
    assert(equal(bat.percentage, 0.5f));
    assert(bat.milliVolts == 300);
  }

  {
    LogTest t{"testBattery charging"};
    Bat::BatteryChecker b{MockExecutor{
        .expectedExecutes =
            {
                {AT::checkBattery},
            },
        .returns =
            {
                {"+CBC: 1,75,450\r\n\r\nOK\r\n"},
            },
    }};
    const auto bat = b.getBattery();
    assert(bat.status == Bat::Battery::Status::Charging);
    assert(equal(bat.percentage, 0.75f));
    assert(bat.milliVolts == 450);
  }

  {
    LogTest t{"testBattery full"};
    Bat::BatteryChecker b{MockExecutor{
        .expectedExecutes =
            {
                {AT::checkBattery},
            },
        .returns =
            {
                {"+CBC: 2,100,600\r\n\r\nOK\r\n"},
            },
    }};
    const auto bat = b.getBattery();
    assert(bat.status == Bat::Battery::Status::Full);
    assert(equal(bat.percentage, 1.0f));
    assert(bat.milliVolts == 600);
  }
}

int main(int argc, char *argv[]) {
  testCall();
  testSMS();
  testNetwork();
  testBattery();
  return 0;
}
