#include "debug.h"
#include "human.h"
#include "machine.h"

#include <cassert>
#include <iomanip>
#include <iostream>
#include <vector>

using namespace PoCo;

struct LogTest {
  LogTest(std::string ctx) : ctx(std::move(ctx)) {
    std::cout << "[" << this->ctx << "]\n";
  }
  ~LogTest() { std::cout << "[" << ctx << "] (PASS)\n"; }

private:
  std::string ctx;
};

struct Examples {
  Position p1{46.7624859, 18.6304591};
  Position p2{47.1258945, 17.8372091};
  CarLocation car{p1, 250.25, 0.8912, 1670077542109ll};
  ParkLocation park{p2, 500.25};
  Status statusPD{Status::Type::ParkingDetected};
  Status statusPU{Status::Type::ParkingUpdated};
  Status statusCD{Status::Type::CarTheftDetected};
  Service sOn{true};
  Service sOff{false};
};

auto testDataProtector() {
  Examples e;
  std::vector<Protector> testData{
      Protector{std::nullopt, std::nullopt, std::nullopt, std::nullopt},
      Protector{e.car, std::nullopt, std::nullopt, std::nullopt},
      Protector{std::nullopt, e.park, std::nullopt, std::nullopt},
      Protector{std::nullopt, std::nullopt, e.statusCD, std::nullopt},
      Protector{std::nullopt, std::nullopt, std::nullopt, e.sOn},
      Protector{std::nullopt, e.park, e.statusPU, std::nullopt},
      Protector{e.car, e.park, e.statusPD, e.sOn},
  };
  return testData;
}

void testMachineSerializationProtector() {
  LogTest l{"testMachineSerializationProtector"};
  Machine::ProtectorS s;
  for (const auto &o : testDataProtector()) {
    const auto str = s.serialize(o);
    std::cout << "  test: " << to_string(o) << "\n        " << str << "\n";
    Protector p;
    assert(s.deserialize(p, str));
    assert(to_string(p) == to_string(o));
  }
}

void testMachineSerializationWatcher() {
  LogTest l{"testMachineSerializationWatcher"};

  std::vector<Watcher> testData{
      Watcher{std::nullopt, std::nullopt, std::nullopt, std::nullopt,
              std::nullopt},
      Watcher{Call{true}, std::nullopt, std::nullopt, std::nullopt,
              std::nullopt},
      Watcher{Call{false}, Refresh{false}, std::nullopt, std::nullopt,
              std::nullopt},
      Watcher{std::nullopt, Refresh{true}, Park{false}, std::nullopt,
              std::nullopt},
      Watcher{std::nullopt, std::nullopt, Park{true},
              ReceiverInfo{ReceiverInfo::Type::Gcm, "phonenumber"},
              std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::Service, "phonenumber"},
              Service{false}},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsHuman, "phonenumber"},
              Service{true}},
      Watcher{Call{false}, Refresh{false}, Park{true},
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "phonenumber"},
              Service{false}},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, ""}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, " "}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "  "}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "   "},
              std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, ";"}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, ";;"}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, ";;;"},
              std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, ";;;;"},
              std::nullopt},
// TODO: escaping _ is not working, check whether it is possible to fix it in backward compatible way
//      Watcher{std::nullopt, std::nullopt, std::nullopt,
//              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "_"}, std::nullopt},
//      Watcher{std::nullopt, std::nullopt, std::nullopt,
//              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "__"}, std::nullopt},
//      Watcher{std::nullopt, std::nullopt, std::nullopt,
//              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "___"},
//              std::nullopt},
//      Watcher{std::nullopt, std::nullopt, std::nullopt,
//              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "____"},
//              std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "*"}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "**"}, std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "***"},
              std::nullopt},
      Watcher{std::nullopt, std::nullopt, std::nullopt,
              ReceiverInfo{ReceiverInfo::Type::SmsMachine, "****"},
              std::nullopt},
      Watcher{
          std::nullopt, std::nullopt, std::nullopt,
          ReceiverInfo{ReceiverInfo::Type::SmsMachine, " t f * ; / ? / ** "},
          std::nullopt},
  };

  Machine::WatcherS s;
  for (const auto &o : testData) {
    const auto str = s.serialize(o);
    std::cout << "  test: " << to_string(o) << "\n        " << str << "\n";
    Watcher w;
    assert(s.deserialize(w, str));
    std::cout << "to_string(w): " << to_string(w) << "\n";
    std::cout << "to_string(e): " << to_string(o) << "\n";
    assert(to_string(w) == to_string(o));
  }
}

void testHumanSerializationProtector() {
  LogTest l{"testHumanSerializationProtector"};

  using namespace std::string_literals;
  std::vector<std::string> expected{
      ""s,

      "http://maps.google.com/?q=46.7624859,18.6304591\n\n"
      "250.25 meters, 89.12 %, 12/03/22 15:25:42 CET\n\n"s,

      "Last park location\n\n"
      "http://maps.google.com/?q=47.1258945,17.8372091\n\n"
      "500.25 meters\n\n"s,

      ""s,

      "Service on\n\n"s,

      "Last park location\n\n"
      "http://maps.google.com/?q=47.1258945,17.8372091\n\n"
      "500.25 meters\n\n"s,

      "http://maps.google.com/?q=46.7624859,18.6304591\n\n"
      "250.25 meters, 89.12 %, 12/03/22 15:25:42 CET\n\n"
      "Service on\n\n"
      "Park distance 72519.74 meters\n\n"s,
  };

  const auto testData = testDataProtector();
  assert(expected.size() == testData.size());

  Human::ProtectorS s;
  int i = 0;
  for (const auto &o : testData) {
    const auto str = s.serialize(o);
    std::cout << "  test: " << to_string(o) << "\n";
    assert(expected[i] == str);
    ++i;
  }
}

void testHumanSerializationWatcher() {
  LogTest l{"testHumanSerializationWatcher"};

  struct TestData {
    std::string commad;
    Watcher expected;
  };

  std::vector<TestData> testData{
      {"location", Watcher{std::nullopt, Refresh{true}, std::nullopt,
                           std::nullopt, std::nullopt}},
      {"call", Watcher{Call{true}, std::nullopt, std::nullopt, std::nullopt,
                       std::nullopt}},
      {"park on", Watcher{std::nullopt, std::nullopt, Park{true}, std::nullopt,
                          std::nullopt}},
      {"park off", Watcher{std::nullopt, std::nullopt, Park{false},
                           std::nullopt, std::nullopt}},
      {"service on", Watcher{std::nullopt, std::nullopt, std::nullopt,
                             std::nullopt, Service{true}}},
      {"service off", Watcher{std::nullopt, std::nullopt, std::nullopt,
                              std::nullopt, Service{false}}},
  };

  Human::WatcherS s;
  for (const auto &o : testData) {
    std::cout << "  test: " << o.commad << "\n        " << to_string(o.expected)
              << "\n";
    Watcher w;
    assert(s.deserialize(w, o.commad));
    assert(to_string(w) == to_string(o.expected));
  }

  Watcher w;
  assert(s.deserialize(w, "invalid") == false);
}

template <typename U>
void serializeConcept(const CSeriaziable<U> auto &s, const U &o) {
  U u;
  [[maybe_unused]] const auto _ = s.deserialize(u, s.serialize(o));
}

void serializeConceptProtector(const CSProtector auto &s, const Protector &o) {
  Protector p;
  [[maybe_unused]] const auto _ = s.deserialize(p, s.serialize(o));
}

void serializeConceptWatcher(const CSWatcher auto &s, const Watcher &o) {
  Watcher w;
  [[maybe_unused]] const auto _ = s.deserialize(w, s.serialize(o));
}

void testConcepts() {
  LogTest l{"testConcepts"};

  // just usage demo
  serializeConcept(Machine::ProtectorS{}, Protector{});
  serializeConcept(Human::ProtectorS{}, Protector{});
  serializeConcept(Machine::WatcherS{}, Watcher{});
  serializeConcept(Human::WatcherS{}, Watcher{});

  serializeConceptProtector(Machine::ProtectorS{}, Protector{});
  serializeConceptProtector(Human::ProtectorS{}, Protector{});
  serializeConceptWatcher(Machine::WatcherS{}, Watcher{});
  serializeConceptWatcher(Human::WatcherS{}, Watcher{});
}

int main(int argc, char *argv[]) {
  testMachineSerializationProtector();
  testMachineSerializationWatcher();
  testHumanSerializationProtector();
  testHumanSerializationWatcher();
  testConcepts();
  return 0;
}
