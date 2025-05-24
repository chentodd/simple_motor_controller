use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;

use mpu6050_dmp::{
    accel::{Accel, AccelFullScale},
    address::Address,
    calibration::ReferenceGravity,
    gyro::{Gyro, GyroFullScale},
    sensor_async::Mpu6050,
};
use postcard_rpc::server::Sender;

use crate::communication::communication::AppTx;
use protocol::*;

// mpu6050
const ACCEL_SCALE: AccelFullScale = AccelFullScale::G2;
const GYRO_SCALE: GyroFullScale = GyroFullScale::Deg2000;
const REF_GRAVITY: ReferenceGravity = ReferenceGravity::ZN;
const MPU_6050_SAMPLE_PERIOD: f32 = 0.05;

// These values are obtained from the calibration process
const ACCEL_CALIBRATION: (i16, i16, i16) = (-2453, -3243, -1793);
const GYRO_CALIBRATION: (i16, i16, i16) = (133, 32, -59);

#[embassy_executor::task]
pub async fn mpu6050_data_publish_task(app_sender: Sender<AppTx>, i2c: I2c<'static, Async>) {
    let mut mpu6050_topic_seq = 0_u8;
    let mut mpu6050_motion_data = Mpu6050MotionData::default();

    // MPU6050 init
    let mut mpu6050 = Mpu6050::new(i2c, Address::default()).await.unwrap();

    let mut delay = embassy_time::Delay;
    mpu6050.initialize_dmp(&mut delay).await.unwrap();

    mpu6050.set_accel_full_scale(ACCEL_SCALE).await.unwrap();
    mpu6050.set_gyro_full_scale(GYRO_SCALE).await.unwrap();
    mpu6050
        .set_accel_calibration(&Accel::new(
            ACCEL_CALIBRATION.0,
            ACCEL_CALIBRATION.1,
            ACCEL_CALIBRATION.2,
        ))
        .await
        .unwrap();
    mpu6050
        .set_gyro_calibration(&Gyro::new(
            GYRO_CALIBRATION.0,
            GYRO_CALIBRATION.1,
            GYRO_CALIBRATION.2,
        ))
        .await
        .unwrap();

    #[cfg(feature = "calibrate-mpu")]
    {
        use defmt::info;
        use mpu6050_dmp::calibration::CalibrationParameters;

        info!("Calibrating mpu");

        let calibration_params = CalibrationParameters::new(ACCEL_SCALE, GYRO_SCALE, REF_GRAVITY);
        let _data = mpu6050
            .calibrate(&mut delay, &calibration_params)
            .await
            .unwrap();

        // If new calibration process is run, the `data` can be printed and update `ACCEL_CALIBRATION`
        // and `GYRO_CALIBRATION` constants.
    }

    let sample_rate = Hertz((1.0 / MPU_6050_SAMPLE_PERIOD) as u32); // 100Hz (1000Hz / (1 + 9))
    let divider = (1000 / sample_rate.0 - 1) as u8;
    mpu6050.set_sample_rate_divider(divider).await.unwrap();

    let _ = REF_GRAVITY;
    loop {
        let (accel, gyro) = mpu6050.motion6().await.unwrap();
        let accel = accel.scaled(ACCEL_SCALE);
        let gyro = gyro.scaled(GYRO_SCALE);
        mpu6050_motion_data.acc_x = accel.x();
        mpu6050_motion_data.acc_y = accel.y();
        mpu6050_motion_data.acc_z = accel.z();
        mpu6050_motion_data.g_x = gyro.x();
        mpu6050_motion_data.g_y = gyro.y();
        mpu6050_motion_data.g_z = gyro.z();

        let _ = app_sender
            .publish::<Mpu6050MotionDataTopic>(mpu6050_topic_seq.into(), &mpu6050_motion_data)
            .await;

        mpu6050_topic_seq = mpu6050_topic_seq.wrapping_add(1);
        Timer::after_millis(sample_rate.0 as u64).await;
    }
}
