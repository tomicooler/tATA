# tATA
the Anti Theft Auto - Pico SIM868

remaking my old android based app

TODO: proper readme

I'll try some new C++ features along the way, e.g.: concepts: https://www.youtube.com/watch?v=gTNJXVmuRRA&t=135s


```
mkdir build; cd build && cmake -DBUILD_FOR_PICO=OFF ../pico/ && make
mkdir build; cd build && cmake ../pico && make && make test
```

```
$tATA/location/12345
$tATA/call/12345
$tATA/park [on/off]/12345
$tATA/service [on/off]/12345
```
