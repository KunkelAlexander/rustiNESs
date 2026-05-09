use crate::{interfaces::{ApuInterface}};



fn fast_sin(t: f32) -> f32{
    let mut j = t * 0.15915;
    j = j - j.floor();
    20.785 * j * (j - 0.5) * (j - 1.0)
}

pub struct OscillatorPulse {
    frequency:   f32, 
    duty_cycle:  f32, 
    amplitude:   f32, 
    harmonics:   usize, 
}

impl OscillatorPulse {

    pub fn new() -> Self {
        Self {
            frequency:  0., 
            duty_cycle: 0., 
            amplitude:  1., 
            harmonics:  20, 
        }
    }

    // Fourier-transform of pulse-wave
    // Nesdev explains how to conver the pulse square waves to these nicer Fourier transforms
    // https://www.nesdev.org/wiki/APU_Pulse
    // Depending on the output video mode, the pulse square waves will look different
    // We can instead calculate the frequency and sample this nicer Fourier transform
    pub fn sample(&self, t: f32) -> f32 {
        let mut a = 0.0;
        let mut b = 0.0; 
        let pi    = std::f32::consts::PI; 
        let p     = self.duty_cycle * 2.0 * pi;

        for n in 1..self.harmonics {
            let n = n as f32;

            let c = n * self.frequency * 2.0 * pi * t;
            a += -fast_sin(c) / n; 
            b += -fast_sin(c - p * n) / n; 
        } 

        (2.0 * self.amplitude / pi) * (a - b)
    }

    

}

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
                SequencerKind::Pulse    => {
                    // This sounds terrible!
                    self.sequence = self.sequence.rotate_right(1)
                }
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
    pulse1_enable:       bool, 
    pulse1_sample:       f32,
    pulse1_sequence:     Sequencer, 
    pulse1_osc:          OscillatorPulse,
    clock_counter:       u32, 
    frame_clock_counter: u32, 
    global_time:         f64,
}


impl Olc2A03 {
    pub fn new() -> Self {
        Self {
            pulse1_enable:       true, 
            pulse1_sample:       0.0,
            pulse1_sequence:     Sequencer::new(SequencerKind::Pulse), 
            pulse1_osc:          OscillatorPulse::new(),
            clock_counter:       0, 
            frame_clock_counter: 0, 
            global_time:         0.0

        }
    }

    pub fn clock(&mut self) {
        let mut is_quarter_frame_clock = false; 
        let mut is_half_frame_clock    = false; 

        self.global_time += 1. / 1789773. / 3.;

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

            
            // The binary output of the sequencer sounds terrible
            //self.pulse1_sample = self.pulse1_sequence.clock(self.pulse1_enable) as f32; 

            self.pulse1_osc.frequency = 1789773. / (16. * ((self.pulse1_sequence.reload as f32) + 1.));
            self.pulse1_sample        = self.pulse1_osc.sample(self.global_time as f32); 
        }

        self.clock_counter        = self.clock_counter.wrapping_add(1);
    }

    


    pub fn reset(&self) {

    }

    // Perform the mixing
    pub fn get_output_sample(&self) -> f32 {
        self.pulse1_sample as f32
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
                    0x00 => {self.pulse1_sequence.sequence = 0b00000001; self.pulse1_osc.duty_cycle = 0.125;},
                    0x01 => {self.pulse1_sequence.sequence = 0b00000011; self.pulse1_osc.duty_cycle = 0.250;},
                    0x02 => {self.pulse1_sequence.sequence = 0b00001111; self.pulse1_osc.duty_cycle = 0.500;},
                    0x03 => {self.pulse1_sequence.sequence = 0b11111100; self.pulse1_osc.duty_cycle = 0.750;},
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