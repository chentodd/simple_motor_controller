## Description

Record embedded rust learnings with nucleo f401re development board

## TODOS

- [x] Read encoder values from motor
- [x] Test pwm, drive motor with fixed duty cycle
- [x] Test pid, control motor velocity
- [x] Include `micropb`
- [x] Define needed protobuf message to communicate with board
- [ ] Implement and test PID auto tuning
- [x] Check S-curve motion interpolation, implement it on the board
- [ ] Test IMU, read IMU settings
- [ ] Create UI that reads controlled data and sends command to the board $$\to$$ working
- [ ] Add a position control loop to minimize position error between actual position and interpolated position

