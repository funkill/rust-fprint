use fprint_rs::{EnrollResult, FPrint, FPrintError, Finger};
use std::{
    io::{stdin, Read},
};

fn main() -> Result<(), FPrintError> {
    println!(
        "This program will enroll your right index finger, \
         unconditionally overwriting any right-index print that was enrolled \
         previously. If you want to continue, press enter, otherwise hit \
         Ctrl+C"
    );

    let _ = stdin().read(&mut [0u8]);

    let fprint = FPrint::new()?;
    let discovered = fprint.discover();
    let device = discovered.get(0).expect("Device not found").open();

    println!(
        "You will need to successfully scan your finger {} times to complete the process.",
        device.get_nr_enroll_stages()
    );

    let mut counter = 1;
    let print_data = loop {
        println!("Scan your finger now (time: {}).", counter);
        let enroll = device.enroll_finger_image()?;
        match enroll {
            EnrollResult::Complete(print, _) => {
                println!("Enroll complete!");
                break print;
            },
            EnrollResult::Fail => println!("Enroll failed, something wen't wrong :("),
            EnrollResult::Pass(_) => {
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
        }
    };

    println!("Enrollment completed!");

    print_data.save_to_disk(Finger::RightIndex)?;
    println!("Print data saved");

    Ok(())
}
