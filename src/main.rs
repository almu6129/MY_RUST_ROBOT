
 #![no_std]     //Don't use the standard library
 #![no_main]    //Tells compliler not to look for a normal main function starting point
 
 use arduino_hal::prelude::*;
 use panic_halt as _;       //Setting up a panic handler
 
 #[arduino_hal::entry]      //Setting up a entry point for the program


 fn main() -> ! {                   //Our makeshift main function


     let dp = arduino_hal::Peripherals::take().unwrap();        //Taking and unwrapping our peripheral objects

     let pins = arduino_hal::pins!(dp);          //Getting the pins from our peripheral objects
 
     // Digital pin 13 is also connected to an onboard LED
     let mut led = pins.d13.into_output();              

     //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
     let mut serial = arduino_hal::default_serial!(dp, pins, 57600);            //Creating a Serial communication object

     let mut trig = pins.d2.into_output();              //Initializing and setting the directions of the arudiunos pins we are using
    let echo = pins.d3; // pin is input by default

    let timer1 = dp.TC1;                //initializing a timer object
    timer1.tccr1b.write(|w| w.cs1().prescale_64());
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        // Important because this sets the bit in the DDR register!
        pins.d9.into_output();

        // - TC1 runs off a 250kHz clock, with 5000 counts per overflow => 50 Hz signal.
        // - Each count increases the duty-cycle by 4us.
        // - Use OC1A which is connected to D9 of the Arduino Uno.
        // let tc1 = dp.TC1;
        // tc1.icr1.write(|w| unsafe { w.bits(4999) });
        // tc1.tccr1a
        // .write(|w| w.wgm1().bits(0b10).com1a().match_clear());
        //  tc1.tccr1b
        // .write(|w| w.wgm1().bits(0b11).cs1().prescale_64());

        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        led.set_high();   //Initially setting the LED to hight

         led.toggle();
         arduino_hal::delay_ms(500);        //Blinking the LED a couple times to know our program uploaded
         led.toggle();
         arduino_hal::delay_ms(500);

         led.toggle();
         arduino_hal::delay_ms(500);
         led.toggle();
         arduino_hal::delay_ms(500);
         
         //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
         //This section contains the main loop. The first part of the main loop holds code for the ultra sonic sensor
         'outer: loop {
            // the timer is reinitialized with value 0.
            timer1.tcnt1.write(|w| unsafe { w.bits(0) });
    
            // the trigger must be set to high under 10 µs as per the HC-SR04 datasheet
            trig.set_high();
            arduino_hal::delay_us(10);
            trig.set_low();
    
            while echo.is_low() {
                // exiting the loop if the timer has reached 200 ms.
                // 0.2s/4µs = 50000
                if timer1.tcnt1.read().bits() >= 50000 {
                    // jump to the beginning of the outer loop if no obstacle is detected
                    ufmt::uwriteln!(
                        &mut serial,
                        "Nothing was detected and jump to outer loop.\r"
                    )
                    .void_unwrap();
                    continue 'outer;          // Jumping back to the start of the loop
                }
            }
            // Restarting the timer
            timer1.tcnt1.write(|w| unsafe { w.bits(0) });
    
            // Wait for the echo to get low again
            while echo.is_high() {}
    
            // 1 count == 4 µs, so the value is multiplied by 4.
            // 1/58 ≈ (34000 cm/s) * 1µs / 2
            // when no object is detected, instead of keeping the echo pin completely low,
            // some HC-SR04 labeled sensor holds the echo pin in high state for very long time,
            // thus overflowing the u16 value when multiplying the timer1 value with 4.
            // overflow during runtime causes panic! so it must be handled
            let temp_timer = timer1.tcnt1.read().bits().saturating_mul(4);
            let value = match temp_timer {
                u16::MAX => {
                    ufmt::uwriteln!(
                        &mut serial,
                        "Nothing was detected and jump to outer loop.\r"            //In the case we don't get anything back after setting echo high
                    )
                    .void_unwrap();
                    continue 'outer;          // Jumping back to the start of the loop
                }
                _ => temp_timer / 58,
            };
    
            // Await 100 ms before sending the next trig
            // 0.1s/4µs = 25000
            while timer1.tcnt1.read().bits() < 25000 {}
    
            ufmt::uwriteln!(
                &mut serial,
                "Hello, we are {} cms away from target!\r",             //A method to print out the distance in a formatted manner
                value
            )
            .void_unwrap(); 

            //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            //The code below is for the servo motor

            // 100 counts => 0.4ms
            // 700 counts => 2.8ms
            // for duty in 100..=700 {
            // timer1.ocr1a.write(|w| unsafe { w.bits(duty) });
            // arduino_hal::delay_ms(20);

            //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


        }
    
    }
}

   