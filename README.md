## Description

This branch is used to utilize the power of `postcard-rpc` and create more reliable communication between host and target.

The `serial` branch contains old stuff that uses serial communication between host and target.

A small project that controls motor in velocity and position mode:
1. `fw` contains the code for stm32f303 discovey board
    * PID is used to control velocity
    * S-curve interpolation is used to control position. The interpolation will calculate needed velocity command
    and send it to PID
    * The motor will be halted if connection is broken
2. `tuning_tool` contains the code for UI:
    * Connect to the board through USB and communicate with `postcard` protocol
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

- [ ] Implement and test PID auto tuning
- [ ] Add a position control loop to minimize position error between actual position and interpolated position
- [ ] Check lookahead processing, implement it to give better support to position command

## Others

I'm new to Rust embedded. If you find any issues, suggestions, improvements, please feel free to open a issue directly, thanks!!

