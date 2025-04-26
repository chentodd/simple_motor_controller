## Description

**Below description is out of date, I'll update it in the future**

A small project that controls motor in velocity and position mode:
1. `fw` contains the code for nucleo f401re development board
2. `serial_tool` contains the code for UI: (it will be updated after including `postcard-rpc`)
    * Connect to the board through serial port and communicate with proto messages
    * Send velocity and position commands to the board to control motor
        - Velocity commands, directly set the reference of PID velocity control loop in the board
        - Position commands, run S-curve interpolation in the board and feed interpolated velocity to PID velocity control loop
    * Display motion profile values:
        - Common, for velocity mode and position mode
          - act pos (unit: rad)
          - act vel (unit: rpm)
        - Position mode only
          - intp pos (unit: rad)
          - intp vel (unit: rad/s)
          - intp acc (unit: rad/s^2)
          - intp jerk (unit: rad/s^3)

## TODOS

A new development board(stm32f303 discovery) is used to test `postcard-rpc` + USB, some tasks need to be revisited. And here are some notes
that I found when debugging the board with `postcard-rpc`:
1. `embassy` example can be used to create a raw USB in firmware
2. `postcard-rpc` firmware example can be used to create a template (note, make sure `usb` task is running after downloading the code)

- [ ] Update UI that reads controlled data and sends command to the board using `postcard-rpc` $\to$ working
- [ ] Test IMU, read IMU settings $\to$ working
- [ ] Implement and test PID auto tuning
- [ ] Add a position control loop to minimize position error between actual position and interpolated position

## Others

I'm new to Rust embedded. If you find any issues, suggestions, improvements, please feel free to open a issue directly, thanks!!

