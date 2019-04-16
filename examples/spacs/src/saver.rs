#![feature(generators, generator_trait)]
mod common;

use failure::Error;
use fprint_rs::{Device, EnrollResult, FPrint, PrintData};
use rusqlite::ToSql;
use std::{
    io::{stdin, Read},
    ops::{Generator, GeneratorState},
    pin::Pin,
};

fn get_user_id() -> Result<u32, Error> {
    let user_id = std::env::args()
        .into_iter()
        .collect::<Vec<_>>()
        .get(1)
        .expect("User id not found (must be first argument)")
        .parse()?;

    Ok(user_id)
}

fn main() -> Result<(), Error> {
    let user_id = get_user_id()?;
    println!(
        "This program will enroll your right index finger, \
         unconditionally overwriting any right-index print that was enrolled \
         previously. If you want to continue, press enter, otherwise hit \
         Ctrl+C"
    );

    let _ = stdin().read(&mut [0u8]);

    let fprint = FPrint::new()?;
    let discovered = fprint.discover()?;
    let device = discovered.get(0).expect("Device not found").open();

    let print_data = enroll_finger(device)?;
    save(print_data, user_id)?;

    Ok(())
}

fn enroll_finger(device: Device) -> Result<PrintData, Error> {
    println!(
        "You will need to successfully scan your finger {} times to complete the process.",
        device.get_nr_enroll_stages()
    );

    let mut enroll = device.enroll();
    let mut counter = 1;
    let print_data = loop {
        println!("Scan your finger now (time: {}).", counter);
        match Pin::new(&mut enroll).resume() {
            GeneratorState::Yielded(state) => match state {
                EnrollResult::Complete => println!("Enroll complete!"),
                EnrollResult::Fail => println!("Enroll failed, something wen't wrong :("),
                EnrollResult::Pass => {
                    println!("Enroll stage passed. Yay!");
                    counter += 1;
                }
                EnrollResult::Retry => println!("Didn't quite catch that. Please try again."),
                EnrollResult::RetryTooShort => {
                    println!("Your swipe was too short, please try again.")
                }
                EnrollResult::RetryCenterFinger => println!(
                    "Didn't catch that, please center your finger on the sensor and try again."
                ),
                EnrollResult::RetryRemoveFinger => {
                    println!("Scan failed, please remove your finger and then try again.")
                }
            },
            GeneratorState::Complete(result) => {
                break result;
            }
        }
    };
    println!("Enrollment completed!");

    Ok(print_data?)
}

fn save(data: PrintData, user_id: u32) -> Result<(), Error> {
    let conn = rusqlite::Connection::open(common::DB_PATH)?;
    let mut stmt =
        conn.prepare("INSERT INTO fingers (user_id, finger, size_data) VALUES (?, ?, ?)")?;
    let data = data.as_bytes()?;
    stmt.execute(&[&user_id as &ToSql, &data, &(data.len() as u32) as &ToSql])?;

    println!("Print data saved");

    Ok(())
}
