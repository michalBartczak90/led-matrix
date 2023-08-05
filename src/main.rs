#![no_std]
#![no_main]

mod characters;

use arduino_hal::hal::port::{PB1, PB0, PB3, Dynamic};
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use panic_halt as _;
use crate::characters::{ CHARACTERS};

const MATRIX_WIDTH: usize = 8;
const LETTER_SPACING: usize = 1;
const SPEED: usize = 25;

const SENTENCE: &str = "Hello world";
const SENTENCE_LED_SIZE: usize = 50; // sum of all characters size ... min :MATRIX_WIDTH

const SENTENCE_REGISTER_SIZE: usize = SENTENCE_LED_SIZE + LETTER_SPACING * (SENTENCE.len() - 1);

// ROWS 74HC595N pins;
type DATA_PIN = Pin<Output, PB3>;
// serial data input
type CLK_PIN = Pin<Output, PB0>;
// shift register clock input
type LATCH_PIN = Pin<Output, PB1>;  // storage register clock input

fn update_column_shift_register(
    data_pin: &mut DATA_PIN,
    clock_pin: &mut CLK_PIN,
    latch_pin: &mut LATCH_PIN,
    data: &u8,
) {
    latch_pin.set_low();
    for i in 0..8 {
        let n = data & (1 << i);

        if n == 0 {
            data_pin.set_low();
        } else {
            data_pin.set_high();
        }
        clock_pin.set_high();
        clock_pin.set_low();
    }
    latch_pin.set_high();
}

fn display_matrix(
    cols: &mut [Pin<Output, Dynamic>; 8],
    data_pin: &mut DATA_PIN,
    clock_pin: &mut Pin<Output, PB0>,
    latch_pin: &mut Pin<Output, PB1>,
    matrix_buffer: &[u8; 8],
) {
    for col_idx in 0..MATRIX_WIDTH {
        cols.iter_mut().for_each(|x| x.set_high());

        let col_data = matrix_buffer.get(col_idx).unwrap_or(&0);
        update_column_shift_register(data_pin, clock_pin, latch_pin, col_data);

        cols[col_idx].set_low();

        arduino_hal::delay_us(500);
    }
}

fn sentence_to_register() -> [u8; SENTENCE_REGISTER_SIZE] {
    let mut register = [0; SENTENCE_REGISTER_SIZE];
    let mut idx = 0;

    for symbol in SENTENCE.chars() {
        let led_symbol = match CHARACTERS.iter().find(|&x| x.symbol == symbol) {
            Some(x) => x.led,
            None => &[0]
        };
        let led_symbol_size = led_symbol.len();
        register[idx..(idx + led_symbol_size)].copy_from_slice(&led_symbol);
        idx += led_symbol_size + LETTER_SPACING;
    }
    register
}


#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::hal::pins!(peripherals);

    // COLS
    let col0 = pins.pd2.into_output().downgrade();
    let col1 = pins.pd3.into_output().downgrade();
    let col2 = pins.pd4.into_output().downgrade();
    let col3 = pins.pd5.into_output().downgrade();
    let col4 = pins.pd6.into_output().downgrade();
    let col5 = pins.pd7.into_output().downgrade();
    let col6 = pins.pc5.into_output().downgrade();
    let col7 = pins.pc4.into_output().downgrade();

    let mut cols = [col7, col6, col5, col4, col3, col2, col1, col0];

    // ROWS
    let mut master_reset = pins.pb4.into_output();
    let mut output_enable = pins.pb2.into_output();
    let mut clk_pin = pins.pb0.into_output();
    let mut latch_pin = pins.pb1.into_output();
    let mut data_pin = pins.pb3.into_output();

    master_reset.set_high();
    output_enable.set_high();
    output_enable.set_low();

    let mut shift_register: [u8; SENTENCE_REGISTER_SIZE] = sentence_to_register();

    for idx in 0..shift_register.len() {
        shift_register[idx] = shift_register[idx].reverse_bits();
    }

    let mut matrix_buffer: [u8; MATRIX_WIDTH] = [0, 0, 0, 0, 0, 0, 0, 0];

    loop {
        shift_register.rotate_left(1);
        matrix_buffer.copy_from_slice(&shift_register[0..8]);

        for _ in 0..SPEED {
            display_matrix(&mut cols, &mut data_pin, &mut clk_pin, &mut latch_pin, &matrix_buffer);
        }
    }
}

