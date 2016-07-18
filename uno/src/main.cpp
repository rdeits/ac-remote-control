/**
 * Blink
 *
 * Turns on an LED on for one second,
 * then off for one second, repeatedly.
 */
#include "Arduino.h"
#include "delay_x.h"

#define MYUBRR 3      // 230.4k baudrate, F_CPU = 16MHz

// We're using pin 13, which is also connected to the internal LED. That means we need to use Port B, pin 5. See: https://www.arduino.cc/en/Reference/PortManipulation
#define IR_PORT PORTB
#define IR_DDR DDRB
#define IR_PIN 5 // corresponds to Arduino pin 13
// #define TRIGGER_PIN 4 // corresponds to Arduino pin 12
#define TRIGGER_PIN 12

#define set_bit(x,y)  x|=(1<<y)
#define clear_bit(x,y)  x&=~(1<<y)

void setup()
{
  // initialize LED digital pin as an output.
  set_bit(IR_DDR, IR_PIN);
  pinMode(TRIGGER_PIN, INPUT);
  // clear_bit(IR_DDR, TRIGGER_PIN);
  // Serial.begin(115200);
}

void send_one() {
  set_bit(IR_PORT, IR_PIN);
  _delay_us(430);
  clear_bit(IR_PORT, IR_PIN);
  _delay_us(1265);
}

void send_zero() {
  set_bit(IR_PORT, IR_PIN);
  _delay_us(450);
  clear_bit(IR_PORT, IR_PIN);
  _delay_us(420);
}

void send_bit(boolean bit) {
  bit ? send_one() : send_zero();
}

void send_header() {
  set_bit(IR_PORT, IR_PIN);
  _delay_us(3400);
  clear_bit(IR_PORT, IR_PIN);
  _delay_us(1750);
}

void send_footer() {
  set_bit(IR_PORT, IR_PIN);
  _delay_us(440);
  clear_bit(IR_PORT, IR_PIN);
  _delay_us(20000);
}

void send_byte(byte data) {
  for (unsigned int i = 0; i < 8; i++) {
    send_bit(data & 0b00000001);
    data >>= 1;
  }
}

void loop()
{
  while (!digitalRead(TRIGGER_PIN)) {
  }

  send_header();
  send_byte(0x23);
  send_byte(0xcb);
  send_byte(0x26);
  send_byte(0x01);
  send_byte(0x00);
  send_byte(0x24);
  send_byte(0x03);
  send_byte(0x08);
  send_byte(0x00);
  send_byte(0x00);
  send_byte(0x00);
  send_byte(0x00);
  send_byte(0x00);
  send_byte(0x44);
  send_footer();

  _delay_ms(1000);
}