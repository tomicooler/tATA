# General

This project uses [atat](https://github.com/FactbirdHQ/atat/) a no_std crate for parsing AT commands.

## Sim868 documentation

- [SIM800 Series AT Command Manual V1.11.pdf](https://www.waveshare.com/wiki/File:SIM800_Series_AT_Command_Manual_V1.11.pdf)
- [SIM868_Series_GNSS_Application_Note_V1.02.pdf](https://www.waveshare.com/wiki/File:SIM868_Series_GNSS_Application_Note_V1.02.pdf)
- [SIM800_Series_GSM_Location_Application_Note_V1.03.pdf](https://www.waveshare.com/wiki/File:SIM800_Series_GSM_Location_Application_Note_V1.03.pdf)
- [SIM868_Series_Hardware_Design_V1.07.pdf](https://www.waveshare.com/wiki/File:SIM868_Series_Hardware_Design_V1.07.pdf)
- [Sim868](https://www.simcom.com/product/SIM868.html)

### Terminology

1.3 Conventions and abbreviations

GSM engines:

- ME (Mobile Equipment)
- MS (Mobile Station)
- TA (Terminal Adapter)
- DCE (Data Communication Equipment)

Controlling device:

- TE (Terminal Equipment)
- DTE (Data Terminal Equipment)

### AT commands syntax

Command: `AT<cmd><CR>`
Response: `<CR><LF><response><CR><LF>`

1.14.1 Basic syntax

`AT<x><n>` or `AT&<x><n>` where `<x>` is the command `<n>` is the parameter.

1.14.2 S parameter syntax

`ATS<n>=<m>` where `<n>` is the index of S register set, `<m>` is the value.

1.14.3 Extended syntax

- Test: `AT+<x>=?`
- Read: `AT+<x>?`
- Write: `AT+<x>=<...>`
- Execute: `AT+<x>`

1.7.1 Parameter Saving mod

- NO_SAVE: parameter is lost on reboot or no parameter.
- AUTO_SAVE: parameter is kept in NVRAM, won't be lost on reboot.
- AT&W_SAVE: parameter is kept in NVRAM, won't be lost on reboot. Use `AT&W` command.
