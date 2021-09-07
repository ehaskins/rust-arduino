cargo objcopy --bin blinker --target thumbv7m-none-eabi --release -- --output-target=binary image.bin

bossac --arduino-erase --erase --write --verify --boot image.bin
Start-Sleep -Seconds 1
bossac --reset