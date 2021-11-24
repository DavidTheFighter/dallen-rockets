This repo contains all of the code used in my rocketry endeavors. Primarily this contains the controller software for the Engine Controller Unit (ECU) for my torch igniter. Here's a video of it firing:

https://user-images.githubusercontent.com/11826091/143272785-673ac291-b50c-4f49-968c-e99f7756ef18.mp4

Along with the data for that firing:

![Figure_1](https://user-images.githubusercontent.com/11826091/143275239-33ec8086-d6ee-4c45-9318-5141252b3e44.png)

Ideally this repository will also soon house the source code for advanced model rocketry as well, as I'm hoping to reuse as much code as possible across my rocketry projects.

Each of the folders in the repo do this thing:
* canfd-ethernet-transceiver - A C++ PlatformIO project that can translate Ethernet packets and CANFD messages between each communication interface. Because my microcontrollers use CANFD, but my laptop cannot, I made this translation layer that sits between them to translate my comms protocol. It's in C++ because the Teensy 4.1 does not have Ethernet support in Rust yet.
* ctrl-send - Used to send commands to a controller
* ctrl-view - Used to receieve data from a controller and display and record it. It also contains a small python script to plot any recorded data.
* ecu - A platform agnostic, no-std controller for a torch igniter (and maybe rocket engine!). This doesn't rely on any set of hardware for better compatbility and flexibility, and to allow for better testing (e.g. easier simulations).
* hal - Contains hardware abstraction layers (HALs) for different systems, such as the hardware the ECU needs to interact with. 
* mcu - Implements the HALs necessary and provides the environment for controller software like the ECU to live when it gets uploaded to a microcontroller. For example, the teensy41-ecu project contains the HAL implementations and environment necessary to run the ECU on a Teensy 4.1 with a custom circuit board attached.

