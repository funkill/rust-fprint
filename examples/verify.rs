use fprint_rs::{FPrint, FPrintError, Finger, VerifyResult};
use std::io::{stdin, Read};

fn main() -> Result<(), FPrintError> {
    let fprint = FPrint::new()?;
    let discovered = fprint.discover();
    let device = discovered.get(0).expect("Device not found").open();

    println!("Opened device. Loading previously enrolled right index finger data...");

    let mut data = device.load_data(Finger::RightIndex)?;
    println!("Print loaded. Time to verify!");

    loop {
        loop {
            println!("Scan your finger now.");
            let result = device.verify_finger_image(&mut data)?;
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
