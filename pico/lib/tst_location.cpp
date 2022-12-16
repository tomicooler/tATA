#include <cassert>
#include <iostream>

#include "gpslocator.h"
#include "gsmlocator.h"
#include "locationservice.h"

namespace {
struct LogTest {
  LogTest(std::string ctx) : ctx(std::move(ctx)) {
    std::cout << "[" << this->ctx << "]\n";
  }
  ~LogTest() { std::cout << "[" << ctx << "] (PASS)\n"; }

private:
  std::string ctx;
};

using GPSGSMLocationService =
    LocationService<GPS::Locator<NoopExecutor>, GSM::Locator<NoopExecutor>>;

struct MockLocator1 {
  std::optional<Location> location{};
  [[nodiscard]] std::optional<Location> getLocation() const { return location; }
};

struct MockLocator2 : public MockLocator1 {};

using MockLocatorService = LocationService<MockLocator1, MockLocator2>;

void testLocationService() {
  {
    LogTest t{"testLocationService without locators"};
    MockLocatorService service;
    const auto loc = service.getLocation();
    assert(!loc.has_value());
  }

  {
    LogTest t{"testLocationService with empty locators"};
    MockLocatorService service;
    service.addLocator(MockLocator1{});
    service.addLocator(MockLocator1{});
    service.addLocator(MockLocator2{});
    service.addLocator(MockLocator2{});
    const auto loc = service.getLocation();
    assert(!loc.has_value());
  }

  {
    LogTest t{"testLocationService with locators"};
    MockLocatorService service;
    service.addLocator(MockLocator1{});
    service.addLocator(MockLocator1{});
    service.addLocator(MockLocator2{Location{2.5}});
    service.addLocator(MockLocator2{});
    const auto loc = service.getLocation();
    assert(loc.has_value());
    assert(loc->latitude == 2.5);
  }

  {
    LogTest t{"testLocationService with locators precedence 1"};
    MockLocatorService service;
    service.addLocator(MockLocator1{Location{3.14}});
    service.addLocator(MockLocator1{});
    service.addLocator(MockLocator2{Location{2.5}});
    service.addLocator(MockLocator2{});
    const auto loc = service.getLocation();
    assert(loc.has_value());
    assert(loc->latitude == 3.14);
  }

  {
    LogTest t{"testLocationService with locators precedence 2"};
    MockLocatorService service;
    service.addLocator(MockLocator2{Location{2.5}});
    service.addLocator(MockLocator2{Location{1.9}});
    service.addLocator(MockLocator1{Location{3.14}});
    service.addLocator(MockLocator1{Location{4.5}});
    const auto loc = service.getLocation();
    assert(loc.has_value());
    assert(loc->latitude == 3.14);
  }
}

bool equal(double a, double b, double epsilon = 0.1) {
  return std::fabs(a - b) < epsilon;
}

void testGetDistance() {
  LogTest t{"testGetDistance"};
  const auto dist =
      getDistanceInMeter(46.7624859, 18.6304591, 47.1258945, 17.8372091);
  assert(equal(dist, 72519.7));
}

struct MockExecutor {
  std::vector<std::string> expectedCommands{};
  std::vector<std::string> returns{};
  size_t idx{};

  std::string execute(const std::string &c) {
    std::cout << "command:" << c << "\n";
    assert(idx < expectedCommands.size());
    assert(expectedCommands.size() == returns.size());
    assert(c == expectedCommands[idx]);
    return returns[idx++];
  };
};

void testGPSLocator() {
  {
    LogTest t{"testGPSLocator"};
    GPS::Locator locator{MockExecutor{
        {{AT::gpsPowerON},
         {AT::gpsInfo},
         {AT::gpsInfo},
         {AT::gpsInfo},
         {AT::gpsPowerOFF}},
        {{AT::OK},
         {"+CGNSINF: ,,,,\r\n\r\nOK\r\n"},
         {"+CGNSINF: ,,,,\r\n\r\nOK\r\n"},
         {"+CGNSINF: 1,1,20221212120221.000,46.7624859,18.6304591,329.218,2."
          "20,285.8,1,,2.1,2.3,0.9,,7,6,,,51,,\r\n\r\nOK\r\n"},
         {AT::OK}}}};
    const auto loc = locator.getLocation();
    assert(loc.has_value());
    assert(equal(loc->latitude, 46.7624859));
    assert(equal(loc->longitude, 18.6304591));
    assert(equal(loc->accuracy, 0.0f));
    assert(loc->timestamp == 1670842941l);
  }

  {
    LogTest t{"testGPSLocator max retries reached"};
    MockExecutor ex{{{AT::gpsPowerON}}, {{AT::OK}}};
    for (int i = 0; i < GPS::Locator<MockExecutor>::MaxRetries; ++i) {
      ex.expectedCommands.push_back(AT::gpsInfo);
      ex.returns.push_back("+CGNSINF: ,,,,\r\n\r\nOK\r\n");
    }
    ex.expectedCommands.push_back(AT::gpsPowerOFF);
    ex.returns.push_back(AT::OK);
    GPS::Locator locator{std::move(ex)};
    const auto loc = locator.getLocation();
    assert(!loc.has_value());
  }
  {
    LogTest t{"testGPSLocator error"};
    GPS::Locator locator{MockExecutor{{{AT::gpsPowerON}}, {{"FAIL\r\n"}}}};
    const auto loc = locator.getLocation();
    assert(!loc.has_value());
  }
}

void testGSMLocator() {
  {
    LogTest t{"testGSMLocator"};
    GSM::Locator locator{MockExecutor{
        {{AT::gprsON},
         {AT::setAPN},
         {AT::bringUpWirelessGPRSorCSD},
         {AT::getLocalIPAddress},
         {AT::setBearerAPN},
         {AT::setBearerGPRS},
         {AT::activateBearer},
         {AT::setLBSAddressFree},
         {AT::getLatLong},
         {AT::getLatLong},
         {AT::getLatLong},
         {AT::deactivateBearer},
         {AT::gprsOFF}},
        {{AT::OK},
         {AT::OK},
         {AT::OK},
         {"AT+CIFSR\r\n100.95.173.97\r\n\r\nOK\r\n"},
         {AT::OK},
         {AT::OK},
         {AT::OK},
         {AT::OK},
         {AT::OK},
         {AT::OK},
         {"+CLBS: 0,18.6304591,46.7624859,550,12/12/22,12:02:21\r\n\r\nOK\r\n"},
         {AT::OK},
         {AT::OK}}}};
    const auto loc = locator.getLocation();
    assert(loc.has_value());
    assert(equal(loc->latitude, 46.7624859));
    assert(equal(loc->longitude, 18.6304591));
    assert(equal(loc->accuracy, 550.0f));
    std::cout << "datetime: " << loc->timestamp << "\n";
    assert(loc->timestamp == 1670842941l);
  }
  // TODO more tests
}

} // namespace

int main(int argc, char *argv[]) {
  testLocationService();
  testGetDistance();
  testGPSLocator();
  testGSMLocator();
  return 0;
}
