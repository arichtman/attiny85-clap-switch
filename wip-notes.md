
### WIP notes on flashing

- https://devblogs.microsoft.com/commandline/connecting-usb-devices-to-wsl/
- https://github.com/dorssel/usbipd-win/wiki/WSL-support#usbip-client-tools
- https://duckduckgo.com/?q=wsl+avrdude&t=braveed&ia=web

- https://github.com/Rahix/avr-hal/tree/main/ravedude
- https://github.com/Rahix/avr-hal
- https://github.com/Rahix/avr-hal/issues/250
- https://rahix.github.io/avr-hal/attiny_hal/index.html

- https://www.reddit.com/r/docker/comments/kd5oz7/vs_code_integration_of_privileged_container_to/



```powershell
sl D:\

```


```Powershell
winget install usbipd --accept-source-agreements
# Reload or open new terminal
usbipd list
usbipd bind --busid=<BUSID>
```
