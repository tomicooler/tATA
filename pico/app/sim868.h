#ifndef TATA_SIM868
#define TATA_SIM868

#include <pico.h>
#include <pico/types.h>

#include <string>

namespace Sim868 {

void init();
void powerOnOff();
void ledBlink();

void start();

std::string sendCommand(const std::string &command,
                        const uint64_t timeout = 2000 * 1000);

} // namespace Sim868

#endif // TATA_SIM868
