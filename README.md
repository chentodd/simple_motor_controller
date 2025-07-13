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

## Hardware

The hardware used in the project is as follows:
- motor: Nidec [24H brushless DC motors](https://www.nidec.com/en/product/search/category/B101/M102/S100/NCJ-24H-24-01/). 
This motor has inbuilt driver that allows me to control the speed with 1 PWM signal
- development board: stm32f303 discovery
- sensor: MPU6050

## TODOS

- [ ] Add a position control loop to minimize position error between actual position and interpolated position
- [ ] Check lookahead processing, implement it to give better support to position command

## Others

I'm new to Rust embedded development, and this project is intended to document my learnings and experiments with PID control and motion interpolation.
If you find any issues, suggestions, improvements, please feel free to open a issue directly, thanks!!
