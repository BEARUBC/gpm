## Getting Started with Raspberry Pi

### Install an operating system using Imager
- Use the Raspberry Pi Imager to flash the OS to your SD card. 
    - [Official Installation Guide](https://www.raspberrypi.com/documentation/computers/getting-started.html#raspberry-pi-imager)


### Set Up your Raspberry Pi
- Once you've succesfully flash the OS to your SD card, follow the steps mentioned below to complete setting up the Raspberry Pi
    - [Official Setup Guide](https://www.raspberrypi.com/documentation/computers/getting-started.html#set-up-your-raspberry-pi)

### Enable UART on Raspberry Pi
1. Remove `console=serial, 11520` from `/boot/firmware/cmdline.txt`
2. Disable bluetooth by adding `dtoverlay=pi3-disable-bt` to `/boot/firmware/config.txt`
    - Note: for RPi4 models, add this instead `dtoverlay=disable-bt`
3. Reboot the Raspberry Pi
4. Run the command: `sudo systemctl disable hciuart`

### Enable SPI on Raspberry Pi
1. Add `dtparam=spi=on` to `/boot/firmware/config.txt`