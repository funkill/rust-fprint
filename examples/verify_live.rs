#![feature(generators, generator_trait)]
use fprint_rs::{Device, EnrollResult, FPrint, FPrintError, PrintData, VerifyResult};
use std::{
    io::{stdin, Read},
    ops::{Generator, GeneratorState},
    pin::Pin,
};

fn enroll(device: &Device) -> Result<PrintData, FPrintError> {
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

    print_data
}

fn main() -> Result<(), FPrintError> {
    let fprint = FPrint::new()?;
    let discovered = fprint.discover()?;
    let device = discovered.get(0).expect("Device not found").open();
    println!("Opened device. It's now time to enroll your finger.");

    let mut print_data = enroll(&device)?;

    println!(
        "Normally we'd save that print to disk, and recall it at some \
         point later when we want to authenticate the user who just \
         enrolled. In the interests of demonstration, we'll authenticate \
         that user immediately."
    );

    loop {
        loop {
            println!("Scan your finger now.");
            let result = device.verify_finger_image(&mut print_data)?;
            match result {
                VerifyResult::NoMatch => {
                    println!("NO MATCH!");
                    break;
                }
                VerifyResult::Match => {
                    println!("MATCH!");
                    break;
                }
                VerifyResult::Retry => println!("Scan didn't quite work. Please try again."),
                VerifyResult::RetryTooShort => println!("Swipe was too short, please try again."),
                VerifyResult::RetryCenterFinger => {
                    println!("Please center your finger on the sensor and try again.")
                }
                VerifyResult::RetryRemoveFinger => {
                    println!("Please remove finger from the sensor and try again.")
                }
            }
        }

        println!("Press Enter to verify again or Ctrl+C to cancel.");
        let _ = stdin().read(&mut [0; 0]);
    }
}
