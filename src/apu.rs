use crate::{interfaces::{ApuInterface}};
pub struct Olc2A03 {
    pulse1_enable: bool, 
    pulse1_sample: f32
}


impl Olc2A03 {
    pub fn new() -> Self {
        Self {
            pulse1_enable: true, 
            pulse1_sample: 0.0,
        }
    }

    pub fn clock(&self) {

    }

    pub fn reset(&self) {

    }

    // Perform the mixing
    pub fn get_output_sample(&self) -> f32 {
        self.pulse1_sample
    }
}



impl ApuInterface for Olc2A03 {
    fn read_cpu(&mut self, addr: u16) -> u8 {
    
        let data = match addr {
            0x0000 => 0x00,
            _      => 0x00,
        };
        data
    }

    fn write_cpu(&mut self, addr: u16, data: u8)  {
        match addr {
            0x4000 => {}, 
            0x4001 => {}, 
            0x4002 => {}, 
            0x4003 => {}, 
            0x4004 => {}, 
            0x4005 => {}, 
            0x4006 => {}, 
            0x4007 => {}, 
            0x4008 => {}, 
            0x400C => {}, 
            0x400E => {}, 
            0x4015 => {}, 
            0x400F => {}, 
            _      => {},
        };
    }


}