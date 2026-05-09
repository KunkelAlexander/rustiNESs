use crate::{interfaces::{ApuInterface}};

pub enum SequencerKind {
    Pulse,
    Triangle,
    Noise,
}
pub struct Sequencer {
    kind:     SequencerKind,
    sequence: u32, 
    timer:    u16, 
    reload:   u16, 
    output:   u8, 
}

impl Sequencer {
    pub fn new(sequencer_kind: SequencerKind) -> Self {
        Self {
            kind:     sequencer_kind,
            sequence: 0x00000000, 
            timer:    0x0000, 
            reload:   0x0000, 
            output:   0x00,
        }
    }

    pub fn clock(&mut self, _enable: bool) -> u8 {

        if self.timer == 0x0000 {
            self.timer  = self.reload + 1; 

            match self.kind {
                SequencerKind::Pulse    => self.sequence = self.sequence.rotate_right(1),
                SequencerKind::Triangle => {},
                SequencerKind::Noise    => {},
            }

            self.output = (self.sequence & 0x00000001) as u8; 
        } else {
            self.timer = self.timer - 1; 
        }

        self.output

    }
}

pub struct Olc2A03 {
    pulse1_enable: bool, 
    pulse1_sample: f32,
    pulse1_sequence: Sequencer, 
    clock_counter: u32, 
    frame_clock_counter: u32, 
}


impl Olc2A03 {
    pub fn new() -> Self {
        Self {
            pulse1_enable: true, 
            pulse1_sample:       0.0,
            pulse1_sequence:     Sequencer::new(SequencerKind::Pulse), 
            clock_counter:       0, 
            frame_clock_counter: 0, 

        }
    }

    pub fn clock(&mut self) {
        let mut is_quarter_frame_clock = false; 
        let mut is_half_frame_clock    = false; 

        if self.clock_counter % 6 == 0 {
            self.frame_clock_counter = self.frame_clock_counter.wrapping_add(1);

            // 4-step sequence mode
            match self.frame_clock_counter {
                3729 => {
                    is_quarter_frame_clock = true;
                },
                7457 => {
                    is_quarter_frame_clock = true;
                    is_half_frame_clock    = true;
                },
                11186 => {
                    is_quarter_frame_clock = true;
                },
                14916 => {
                    is_quarter_frame_clock = true;
                    is_half_frame_clock    = true;
                    self.frame_clock_counter = 0; 
                },
                _ => {}
            }

            if is_quarter_frame_clock {

            }

            if is_half_frame_clock {

            }
        }


        self.pulse1_sample = self.pulse1_sequence.clock(self.pulse1_enable) as f32; 

        self.clock_counter = self.clock_counter.wrapping_add(1);

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
            // Set duty cycle of channel 1's pulse wave form
            0x4000 => {
                match (data & 0xC0) >> 6 {
                    0x00 => {self.pulse1_sequence.sequence = 0b00000001;},
                    0x01 => {self.pulse1_sequence.sequence = 0b00000011;},
                    0x02 => {self.pulse1_sequence.sequence = 0b00001111;},
                    0x03 => {self.pulse1_sequence.sequence = 0b11111100;},
                    _    => {}
                }
            }, 
            0x4001 => {}, 
            // Control pulse 1 sequencer reload value - first 8 bits
            0x4002 => {
                self.pulse1_sequence.reload = (self.pulse1_sequence.reload & 0xFF00) | data as u16;
            }, 
            // Control pulse 1 sequencer reload value - second 8 bits
            0x4003 => {
                self.pulse1_sequence.reload = (self.pulse1_sequence.reload & 0x00FF) | (((data & 0x07) as u16) << 8);
                self.pulse1_sequence.timer  = self.pulse1_sequence.reload;
            }, 
            0x4004 => {}, 
            0x4005 => {}, 
            0x4006 => {}, 
            0x4007 => {}, 
            0x4008 => {}, 
            0x400C => {}, 
            0x400E => {}, 
            // Enable and disable pulse 1 sequencer
            0x4015 => {
                self.pulse1_enable = (data & 0x01) != 0;
            }, 
            0x400F => {}, 
            _      => {},
        };
    }


}