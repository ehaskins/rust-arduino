#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate sam3x8e;

use cortex_m_rt::entry;
use sam3x8e::{PIOB, RTT};

fn delay_ms(rtt: &RTT, ms: u32) {
    // We're not considering overflow here, 32 bits can keep about 49 days in ms
    let now = rtt.vr.read().bits();
    let until = now + ms;

    while rtt.vr.read().bits() < until {}
}

unsafe fn blink(piob:& PIOB, rtt: & RTT, count: u32)->(){
    for i in 0..count {
        if count > 1 && i > 0 {
            delay_ms(&rtt, 200);
        }

        piob.sodr.write_with_zero(|w| w.p27().set_bit());

        delay_ms(&rtt, 50);

        piob.codr.write_with_zero(|w| w.p27().set_bit());
    }
}


#[entry]
unsafe fn main() -> ! {
    let p = sam3x8e::Peripherals::take().unwrap();

    // Enable peripheral clocks
    let pmc = p.PMC;
    pmc.pmc_pcer0.write_with_zero(|w| {
        w
        .pid12().set_bit() // PIOB
    }); 

    // Configure RTT resolution to approx. 1ms
    let rtt = p.RTT;
    rtt.mr.write_with_zero(|w| { w.rtpres().bits(0x20) });

    let pioa = p.PIOA;
    let piob = p.PIOB;

    // Configure PIOB pin 27 (LED)
    piob.per.write_with_zero(|w| w.p27().set_bit());
    piob.oer.write_with_zero(|w| w.p27().set_bit());
    piob.pudr.write_with_zero(|w| w.p27().set_bit());

    // Turn off led
    piob.codr.write_with_zero(|w| w.p27().set_bit());

    // pmc.ckgr_mor.write_with_zero(|w| 
    //     w
    //         .key().bits(0x37) // magic
    //         .moscxtst().bits(1) // test for 8 slow cycles
    //         .moscrcen().set_bit() // enable on-chip RC
    //         .moscxten().set_bit() // enable crystal oscillator

    // )

    let ckgr_mor = pmc.ckgr_mor.read();

    let blinks = match ckgr_mor.moscsel().bit(){
        true => 1,
        false => 2
    };

    delay_ms(&rtt, 500);
    blink(&piob, &rtt, blinks);
    delay_ms(&rtt, 500);

    // Configure UART

    pmc.pmc_pcer0.write_with_zero(|w| {
        w
        .pid8().set_bit() // UART
    }); 

    let uart = p.UART;

    // Set clock divisor
    let uart_clock_divisor = 4000000 / (16 * 115200);
    uart.brgr.write(|w| w.cd().bits(uart_clock_divisor as u16));

    // Enable uart rx and tx
    uart.cr.write_with_zero(|w|
         w
         .rxen().set_bit()
         .txen().set_bit());

    // Configure pinmux for UART
    pioa.pdr.write_with_zero(|w| 
        w
        .p8().set_bit() // RX
        .p9().set_bit()); // TX


    let sr = uart.sr.read();
    if sr.txrdy().bit_is_clear() {
        delay_ms(&rtt, 500);
        blink(&piob, &rtt, 3);
        delay_ms(&rtt, 500);
    }

    loop {
        //check uart for received byte

        let sr = uart.sr.read();
        if sr.rxrdy().bit_is_set() {
            let byte = uart.rhr.read().bits() as u8; 

            uart.thr.write_with_zero(|w| w.txchr().bits(byte));

            //blink(&piob, &rtt, 1);
            //delay_ms(&rtt, 50);

        }

        // blink(&piob, &rtt, 1);
        // delay_ms(&rtt, 100);

    }
}
