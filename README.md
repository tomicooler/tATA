# tATA
the Anti Theft Auto - Pico SIM868

remaking my old android based app

TODO: proper readme

I'll try some new C++ features along the way, e.g.: concepts: https://www.youtube.com/watch?v=gTNJXVmuRRA&t=135s


Building & Testing
```
mkdir build; cd build && cmake -DBUILD_FOR_PICO=OFF ../pico/ && make && make test
mkdir build; cd build && cmake -DCMAKE_BUILD_TYPE=Debug ../pico && make && make test
```

SMS commands
```
$tATA/location/12345
$tATA/call/12345
$tATA/park [on/off]/12345
$tATA/service [on/off]/12345
```

Developer workflow with Raspberry Pi 3B
```
sudo nmap -p 22 192.168.50.0/24

ssh pi@RASPBERRY_IP

# use sshfs to mount the project on the pi
mkdir -p /home/pi/pico/tATA
sshfs youruser@HOST_IP:/home/tomi/qt_workspace/pico/tATA /home/pi/pico/tATA

cd ~/pico/tATA

# building on raspberry would require new gcc/clang
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Debug ../pico
make

openocd -f interface/raspberrypi-swd.cfg -f target/rp2040.cfg -c "program app/app.elf verify reset exit"

minicom -b 115200 -o -D /dev/serial0 # for uart
minicom -b 115200 -o -D /dev/ttyACM0 # for usb
```
