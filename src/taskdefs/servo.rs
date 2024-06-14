// ### This file is temporarily commented out
// ### raestro depends on rppal, which won't compile on mac, will figure out to
// ### simulate the env needed for rppal or configure conditional compiliation

// // All tasks operating on the servo motors will live in this file.
// use anyhow::Result;
// use std::convert::TryInto;
// use std::thread;
// use std::time::Duration;

// use raestro::maestro::builder::Builder;
// use raestro::maestro::constants::Baudrate;
// use raestro::maestro::constants::Channel;
// use raestro::maestro::constants::MAX_QTR_PWM;
// use raestro::maestro::constants::MIN_QTR_PWM;
// use raestro::maestro::Maestro;

// pub async fn handle_servo_task(task_code: i32) {
// 	// Initialize raestro instance
// 	let mut maestro: Maestro = Builder::default()
//         .baudrate(Baudrate::Baudrate11520)
//         .block_duration(Duration::from_millis(100))
//         .try_into()
//         .unwrap();

// 	match task_code {
//         GET_POSITION => {
//             println!("Running the get_position example from https://github.com/BEARUBC/raestro/tree/master/examples");
//             get_position(&maestro).unwrap();
//         }
//         SET_ACCERLERATION => {
//             println!("Running the set_acceleration example from https://github.com/BEARUBC/raestro/tree/master/examples");
//             set_acceleration(&maestro).unwrap();
//         }
//         SET_SPEED => {
//             println!("Running the set_speed example from https://github.com/BEARUBC/raestro/tree/master/examples");
//             set_speed(&maestro).unwrap();
//         }
//         SET_TARGET => {
//             println!("Running the set_target exmaple from https://github.com/BEARUBC/raestro/tree/master/examples");
//             set_target(&maestro).unwrap();
//         }
//         STOP_SCRIPT => {
//             println!("Running the stop_script example from https://github.com/BEARUBC/raestro/tree/master/examples");
//             stop_script(&maestro).unwrap();
//         }
//         _ => {
//             println!("Unmatched task, ignoring...")
//         }
//     }
// }

// fn get_position(maestro: &Maestro) -> Result<()> {
//     let channel = Channel::Channel0;
//     let pos_min = MIN_QTR_PWM;
//     let pos_max = MAX_QTR_PWM;

// 	maestro.set_target(channel, pos_min).unwrap();
//     let position = maestro.get_position(channel).unwrap();
//     assert_eq!(position, pos_min);

//     maestro.set_target(channel, pos_max).unwrap();
//     let position = maestro.get_position(channel).unwrap();
//     assert_eq!(position, pos_max);
// 	Ok(())
// }

// fn set_acceleration(maestro: &Maestro) -> Result<()> {
//     let channel = Channel::Channel0;
//     let pos_min = MIN_QTR_PWM;
//     let pos_max = MAX_QTR_PWM;
//     let accel_min = 1u8;
//     let accel_max = 255u8;
//     let sleep_duration = Duration::from_secs(1);
    
//     maestro.set_acceleration(channel, accel_min).unwrap();
//     maestro.set_target(channel, pos_min).unwrap();

//     maestro.set_acceleration(channel, accel_max).unwrap();
//     maestro.set_target(channel, pos_max).unwrap();
//     Ok(())
// }

// fn set_speed(maestro: &Maestro) -> Result<()> {
//     let channel = Channel::Channel0;
//     let pos_min = MIN_QTR_PWM;
//     let pos_max = MAX_QTR_PWM;
//     let speed0 = 10u16;
//     let speed1 = 140u16;
//     let sleep_duration = Duration::from_secs(1);
    
//     maestro.set_speed(channel, speed0).unwrap();
//     maestro.set_target(channel, pos_min).unwrap();

//     maestro.set_speed(channel, speed1).unwrap();
//     maestro.set_target(channel, pos_max).unwrap();
//     Ok(())
// }

// fn set_target(maestro: &Maestro) -> Result<()> {
//     let channel0 = Channel::Channel0;
//     let channel1 = Channel::Channel1;
//     let channel2 = Channel::Channel2;
//     let pos_min = MIN_QTR_PWM;
//     let pos_max = MAX_QTR_PWM;
//     let sleep_duration = Duration::from_secs(1);
    
//     maestro.set_target(channel0, pos_min).unwrap();
//     maestro.set_target(channel1, pos_min).unwrap();
//     maestro.set_target(channel2, pos_min).unwrap();
    
//     maestro.set_target(channel0, pos_max).unwrap();
//     maestro.set_target(channel1, pos_max).unwrap();
//     maestro.set_target(channel2, pos_max).unwrap();
//     Ok(())
// }

// fn stop_script(maestro: &Maestro) -> Result<()> {
//     let channel = Channel::Channel0;
//     let pos_min = MIN_QTR_PWM;
//     let pos_max = MAX_QTR_PWM;
//     let sleep_duration = Duration::from_secs(1);
    
//     maestro.set_target(channel, pos_min).unwrap();
//     maestro.stop_script().unwrap();

//     maestro.set_target(channel, pos_max).unwrap();
//     maestro.stop_script().unwrap();
//     Ok(())
// }