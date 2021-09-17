#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate sam3x8e;

use cortex_m_rt::entry;
use sam3x8e::{EFC0, EFC1, PIOB, RTT};

fn delay_ms(rtt: &RTT, ms: u32) {
  // We're not considering overflow here, 32 bits can keep about 49 days in ms
  let now = rtt.vr.read().bits();
  let until = now + ms;

  while rtt.vr.read().bits() < until {}
}

fn blink(piob: &PIOB, rtt: &RTT, count: u32) -> () {
  for i in 0..count {
    if count > 1 && i > 0 {
      delay_ms(&rtt, 200);
    }

    unsafe {
      piob.sodr.write_with_zero(|w| w.p27().set_bit());
    }

    delay_ms(&rtt, 50);

    unsafe {
      piob.codr.write_with_zero(|w| w.p27().set_bit());
    }
  }
}

#[entry]
unsafe fn main() -> ! {
  let p = sam3x8e::Peripherals::take().unwrap();

  // Enable peripheral clocks
  let pmc = p.PMC;
  pmc.pmc_pcer0.write_with_zero(|w| {
    w.pid12().set_bit() // PIOB
  });

  // Configure RTT resolution to approx. 1ms
  let rtt = p.RTT;
  rtt.mr.write_with_zero(|w| w.rtpres().bits(0x20));

  let pioa = p.PIOA;
  let piob = p.PIOB;

  // Configure PIOB pin 27 (LED)
  piob.per.write_with_zero(|w| w.p27().set_bit());
  piob.oer.write_with_zero(|w| w.p27().set_bit());
  piob.pudr.write_with_zero(|w| w.p27().set_bit());

  // Turn off led
  piob.codr.write_with_zero(|w| w.p27().set_bit());

  // Set main clock to 84mhz
  configure_clock(&pmc, &p.EFC0, &p.EFC1);

  delay_ms(&rtt, 200);
  blink(&piob, &rtt, 2);
  delay_ms(&rtt, 1000);

  // Configure UART

  pmc.pmc_pcer0.write_with_zero(|w| {
    w.pid8().set_bit() // UART
  });

  let uart = p.UART;

  // Set clock divisor
  let uart_clock_divisor = 84000000 / (16 * 115200);
  uart.brgr.write(|w| w.cd().bits(uart_clock_divisor as u16));

  // Enable uart rx and tx
  uart
    .cr
    .write_with_zero(|w| w.rxen().set_bit().txen().set_bit());

  // Configure pinmux for UART
  pioa.pdr.write_with_zero(|w| {
    w.p8()
      .set_bit() // RX
      .p9()
      .set_bit()
  }); // TX

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

fn configure_clock(pmc: &sam3x8e::PMC, efc0: &EFC0, efc1: &EFC1) {
  efc0.fmr.write(|w| unsafe { w.fws().bits(4) });
  efc1.fmr.write(|w| unsafe { w.fws().bits(4) });

  if pmc.ckgr_mor.read().moscsel().bit_is_clear() {
    unsafe {
      // Enable crystal oscillator
      pmc.ckgr_mor.write_with_zero(|w| {
        w.key()
          .bits(0x37) // magic
          .moscxtst()
          .bits(8) // test for 8 slow cycles
          .moscrcen()
          .set_bit() // enable on-chip RC
          .moscxten()
          .set_bit() // enable crystal oscillator
      });
    }

    while pmc.pmc_sr.read().moscxts().bit_is_clear() {}
  }

  // Switch main oscillator from RC to external oscillator
  unsafe {
    pmc.ckgr_mor.write(|w| {
      w.key()
        .bits(0x37)
        .moscxtst()
        .bits(8)
        .moscrcen()
        .set_bit()
        .moscxten()
        .set_bit()
        .moscsel()
        .set_bit()
    });
  }
  while pmc.pmc_sr.read().moscsels().bit_is_clear() {}

  // Switch master clock to main clock
  unsafe {
    pmc.pmc_mckr.write_with_zero(|w| w.css().main_clk());
  }
  while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}

  // Enable and configure PLLA to generate 168mhz (12mhz crystal * 14)
  unsafe {
    pmc.ckgr_pllar.write_with_zero(|w| {
      w.one()
        .set_bit()
        .pllacount()
        .bits(0x3f)
        .diva()
        .bits(1)
        .mula()
        .bits(13) // val + 1 = 14 * 12 = 168mhz
    });
  }
  while pmc.pmc_sr.read().locka().bit_is_clear() {}

  // Configure master clock prescaler to divide 168mhz to 84mhz
  pmc.pmc_mckr.write(|w| w.pres().clk_2());
  while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}

  // Switch master clock source to PLLA
  pmc.pmc_mckr.write(|w| w.css().plla_clk().pres().clk_2());
  while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
}
