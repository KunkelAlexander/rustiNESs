use crate::{interfaces::{ApuInterface}};
pub struct OscillatorPulse {
    frequency:   f32, 
    duty_cycle:  f32, 
    duty_idx:    usize,
    amplitude:   f32, 
    harmonics:   usize, 
    phase:       f32,
}


// This is still too slow
fn fast_sin(t: f32) -> f32{
    let mut j = t * 0.15915;
    j = j - j.floor();
    20.785 * j * (j - 0.5) * (j - 1.0)
}



// We therefore precompute the harmonics
const TABLE_SIZE: usize = 4096; // temporal resolution
const DUTY_CYCLES: [f64; 4] = [0.125, 0.25, 0.5, 0.75];

pub struct WaveTable {
    tables: [[f32; TABLE_SIZE]; 4],
}


impl WaveTable {
    pub fn new(harmonics: usize) -> Self {
        // Compute everything in f64 to reduce rounding errors
        let pi = std::f64::consts::PI;
        let mut tables = [[0.0; TABLE_SIZE]; 4];

        for (duty_idx, &duty) in DUTY_CYCLES.iter().enumerate() {
            let p = duty * 2.0 * pi;

            for i in 0..TABLE_SIZE {
                let t = i as f64 / TABLE_SIZE as f64;  // 0.0 .. 1.0, one full period
                let mut sum = 0.0;

                for n in 1..harmonics {
                    let nf = n as f64;
                    let c  = nf * 2.0 * pi * t;
                    sum += (f64::sin(c - p * nf) - f64::sin(c)) / nf; // We can use proper sin here since we precompute
                }

                tables[duty_idx][i] = (sum * (2.0 / pi)) as f32;
            }
        }

        Self { tables }
    }
}

impl OscillatorPulse {

    pub fn new() -> Self {
        Self {
            frequency:  0., 
            duty_cycle: 0., 
            duty_idx:   0,
            amplitude:  1., 
            harmonics:  20, 
            phase:      0.0,
        }
    }

    // Fourier-transform of pulse-wave
    // Nesdev explains how to conver the pulse square waves to these nicer Fourier transforms
    // https://www.nesdev.org/wiki/APU_Pulse
    // Depending on the output video mode, the pulse square waves will look different
    // We can instead calculate the frequency and sample this nicer Fourier transform
    pub fn sample_slow(&self, t: f32, _table: &WaveTable) -> f32 {
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

    // It turned out that sample_slow was still too slow and would use up 60% of the total simulation time
    // Using wave tables, we precompute the loop over the harmonics and the sine evaluations
    pub fn sample(&self, _t: f32, table: &WaveTable) -> f32 {
        // Convert time → phase index
        let idx   = self.phase * TABLE_SIZE as f32;

        self.amplitude * table.tables[self.duty_idx][idx as usize]
    }

    // Ideally, we would compute phase = (t * freq) % 1.0 in sample
    // But this is very slow
    // So, here we advance the phase in a separate step to reduce the fmod calls
    pub fn advance(&mut self, sample_rate: f32) {
        self.phase += self.frequency / sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }


}

pub struct Envelope {
    start:         bool,
    disable:       bool,
    divider_count: u16,
    volume:        u16,
    output:        u16,
    decay_count:   u16,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            start:         false,
            disable:       false,
            divider_count: 0,
            volume:        0,
            output:        0,
            decay_count:   0,
        }
    }

    pub fn clock(&mut self, loop_flag: bool) {
        if !self.start {
            if self.divider_count == 0 {
                self.divider_count = self.volume;

                if self.decay_count == 0 {
                    if loop_flag {
                        self.decay_count = 15;
                    }
                } else {
                    self.decay_count -= 1;
                }
            } else {
                self.divider_count -= 1;
            }
        } else {
            self.start         = false;
            self.decay_count   = 15;
            self.divider_count = self.volume;
        }

        self.output = if self.disable { self.volume } else { self.decay_count };
    }
}

pub struct LengthCounter {
    counter: u8,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn clock(&mut self, enable: bool, halt: bool) -> u8 {
        if !enable {
            self.counter = 0;
        } else if self.counter > 0 && !halt {
            self.counter -= 1;
        }
        self.counter
    }
}

const LENGTH_TABLE: [u8; 32] = [
     10, 254, 20,  2, 40,  4, 80,  6,
    160,   8, 60, 10, 14, 12, 26, 14,
     12,  16, 24, 18, 48, 20, 96, 22,
    192,  24, 72, 26, 16, 28, 32, 30,
];


#[derive(Clone, Copy)]
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
    pub mode: bool, 
}

impl Sequencer {
    pub fn new(sequencer_kind: SequencerKind) -> Self {
        Self {
            kind:     sequencer_kind,
            sequence: match sequencer_kind { SequencerKind::Noise => 0xDBDB, _ => 0x00000000 },
            timer:    0x0000,
            reload:   0x0000,
            output:   0x00,
            mode:     false,
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
                SequencerKind::Noise    => {
                    let s        = self.sequence as u16;
                    let other    = if self.mode { (s & 0x0040) >> 6 } else { (s & 0x0002) >> 1 };
                    let feedback = (s & 0x0001) ^ other;
                    self.sequence = ((feedback << 14) | ((s & 0x7FFF) >> 1)) as u32;
                },
            }

            self.output = (self.sequence & 0x00000001) as u8; 
        } else {
            self.timer = self.timer - 1; 
        }

        self.output

    }
}

pub struct Sweeper {
    enabled: bool,
    down:    bool,
    reload:  bool,
    shift:   u8,
    timer:   u8,
    period:  u8,
    change:  u16,
    mute:    bool,
}

impl Sweeper {
    pub fn new() -> Self {
        Self {
            enabled: false,
            down:    false,
            reload:  false,
            shift:   0x00,
            timer:   0x00,
            period:  0x00,
            change:  0,
            mute:    false,
        }
    }

    pub fn track(&mut self, target: u16) {
        if self.enabled {
            self.change = target >> self.shift;
            self.mute   = (target < 8) || (target > 0x7FF);
        }
    }

    pub fn clock(&mut self, target: &mut u16, channel: bool) -> bool {
        let mut changed = false;

        if self.timer == 0 && self.enabled && self.shift > 0 && !self.mute {
            if *target >= 8 && self.change < 0x07FF {
                if self.down {
                    *target -= self.change - channel as u16;
                } else {
                    *target += self.change;
                }
                changed = true;
            }
        }

        if self.timer == 0 || self.reload {
            self.timer  = self.period;
            self.reload = false;
        } else {
            self.timer -= 1;
        }

        self.mute = (*target < 8) || (*target > 0x7FF);

        changed
    }
}

pub struct Olc2A03 {
    // Pulse 1 
    pulse1_enable:       bool,
    pulse1_halt:         bool,
    pulse1_sample:       f32,
    pulse1_output:       f32,
    pulse1_sequence:     Sequencer,
    pulse1_osc:          OscillatorPulse,
    pulse1_sweep:        Sweeper,
    pulse1_env:          Envelope,
    pulse1_lc:           LengthCounter,
    pulse1_visual:       u16,

    // Pulse 2
    pulse2_enable:       bool,
    pulse2_halt:         bool,
    pulse2_sample:       f32,
    pulse2_output:       f32,
    pulse2_sequence:     Sequencer,
    pulse2_osc:          OscillatorPulse,
    pulse2_sweep:        Sweeper,
    pulse2_env:          Envelope,
    pulse2_lc:           LengthCounter,
    pulse2_visual:       u16,

    // Noise
    noise_enable:        bool,
    noise_halt:          bool,
    noise_output:        f32,
    noise_sequence:      Sequencer,
    noise_env:           Envelope,
    noise_lc:            LengthCounter,
    noise_visual:        u16,

    wavetable:           WaveTable,
    clock_divider:       u8,
    frame_clock_counter: u32,
    global_time:         f64,
}


impl Olc2A03 {
    pub fn new() -> Self {
        Self {
            //  Pulse 1 
            pulse1_enable:       true, 
            pulse1_halt:         false, 
            pulse1_sample:       0.0,
            pulse1_output:       0.0,
            pulse1_sequence:     Sequencer::new(SequencerKind::Pulse),
            pulse1_osc:          OscillatorPulse::new(),
            pulse1_sweep:        Sweeper::new(),
            pulse1_env:          Envelope::new(),
            pulse1_lc:           LengthCounter::new(),
            pulse1_visual:       0,

            // Pulse 2
            pulse2_enable:       true,
            pulse2_halt:         false,
            pulse2_sample:       0.0,
            pulse2_output:       0.0,
            pulse2_sequence:     Sequencer::new(SequencerKind::Pulse),
            pulse2_osc:          OscillatorPulse::new(),
            pulse2_sweep:        Sweeper::new(),
            pulse2_env:          Envelope::new(),
            pulse2_lc:           LengthCounter::new(),
            pulse2_visual:       0,

            // Noise
            noise_enable:        true,
            noise_halt:          false,
            noise_output:        0.0,
            noise_sequence:      Sequencer::new(SequencerKind::Noise),
            noise_env:           Envelope::new(),
            noise_lc:            LengthCounter::new(),
            noise_visual:        0,

            wavetable:           WaveTable::new(32),
            clock_divider:       0,
            frame_clock_counter: 0,
            global_time:         0.0

        }
    }

    pub fn clock(&mut self) {
        let mut is_quarter_frame_clock = false; 
        let mut is_half_frame_clock    = false; 

        // Run every 6 steps
        if self.clock_divider == 0 {
            self.clock_divider = 5;
            const TIME_STEP: f64 = 1.0 / (1789773.0 * 3.0);
            self.global_time += TIME_STEP;
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
                self.pulse1_env.clock(self.pulse1_halt);
                self.pulse2_env.clock(self.pulse2_halt);
                self.noise_env.clock( self.noise_halt);

            }

            if is_half_frame_clock {
                self.pulse1_lc.clock(self.pulse1_enable, self.pulse1_halt);
                self.pulse2_lc.clock(self.pulse2_enable, self.pulse2_halt);
                self.noise_lc.clock(self.noise_enable, self.noise_halt);
                self.pulse1_sweep.clock(&mut self.pulse1_sequence.reload, false);
                self.pulse2_sweep.clock(&mut self.pulse2_sequence.reload, true);
            }

            
            // The binary output of the sequencer sounds terrible
            //self.pulse1_sample = self.pulse1_sequence.clock(self.pulse1_enable) as f32; 

            self.pulse1_osc.frequency = 1789773. / (16. * ((self.pulse1_sequence.reload as f32) + 1.));
            self.pulse1_osc.amplitude = ((self.pulse1_env.output - 1) as f32) / 16.0;
            self.pulse1_osc.advance(1789773.);
            self.pulse1_sample        = self.pulse1_osc.sample(self.global_time as f32, &self.wavetable); 

            
            self.pulse2_osc.frequency = 1789773. / (16. * ((self.pulse2_sequence.reload as f32) + 1.));
            self.pulse2_osc.amplitude = ((self.pulse2_env.output - 1) as f32) / 16.0;
            self.pulse2_osc.advance(1789773.);
            self.pulse2_sample        = self.pulse2_osc.sample(self.global_time as f32, &self.wavetable); 


			if self.pulse1_lc.counter > 0 && self.pulse1_sequence.timer >= 8 && !self.pulse1_sweep.mute && self.pulse1_env.output > 2 {
				self.pulse1_output += (self.pulse1_sample - self.pulse1_output) * 0.5;
            } else {
                self.pulse1_output = 0.0;
            }
            
			if self.pulse2_lc.counter > 0 && self.pulse2_sequence.timer >= 8 && !self.pulse2_sweep.mute && self.pulse2_env.output > 2 {
				self.pulse2_output += (self.pulse2_sample - self.pulse2_output) * 0.5;
            } else {
                self.pulse2_output = 0.0;
            }

            self.noise_sequence.clock(self.noise_enable);
            if self.noise_lc.counter > 0 && self.noise_env.output > 0 {
                let noise_target = self.noise_sequence.output as f32 * ((self.noise_env.output - 1) as f32 / 16.0);
                self.noise_output += (noise_target - self.noise_output) * 0.25;
            } else {
                self.noise_output = 0.0;
            }

            
            if !self.pulse1_enable {
                self.pulse1_output = 0.0;
            }
            if !self.pulse2_enable  {
                self.pulse2_output = 0.0;
            }
            if !self.noise_enable {
                self.noise_output = 0.0;
            }

            self.pulse1_sweep.track(self.pulse1_sequence.reload);
            self.pulse2_sweep.track(self.pulse2_sequence.reload);

            if self.pulse1_enable && self.pulse1_env.output > 1 && !self.pulse1_sweep.mute {
                self.pulse1_visual = self.pulse1_sequence.reload;
            } else {
                self.pulse1_visual = 2047;
            }

            if self.pulse2_enable && self.pulse2_env.output > 1 && !self.pulse2_sweep.mute {
                self.pulse2_visual = self.pulse2_sequence.reload;
            } else {
                self.pulse2_visual = 2047;
            }

            if self.noise_enable && self.noise_env.output > 1 {
                self.noise_visual = self.noise_sequence.reload;
            } else {
                self.noise_visual = 2047;
            }
        } else {
            self.clock_divider -= 1;
        }
    }

    


    pub fn reset(&mut self) {
        self.pulse1_enable       = true;
        self.pulse1_sample       = 0.0;
        self.pulse1_sequence     = Sequencer::new(SequencerKind::Pulse);
        self.pulse1_osc          = OscillatorPulse::new();
        self.clock_divider       = 0;
        self.frame_clock_counter = 0;
        self.global_time         = 0.0;
    }

    // Perform the mixing
    pub fn get_output_sample(&self) -> f32 {
		((1.0 * self.pulse1_output) - 0.8)  * 0.1 +
        ((1.0 * self.pulse2_output) - 0.8)  * 0.1 +
        ((2.0 * (self.noise_output - 0.5))) * 0.1
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
                    0x00 => {self.pulse1_sequence.sequence = 0b00000001; self.pulse1_osc.duty_cycle = 0.125; self.pulse1_osc.duty_idx = 0;},
                    0x01 => {self.pulse1_sequence.sequence = 0b00000011; self.pulse1_osc.duty_cycle = 0.250; self.pulse1_osc.duty_idx = 1;},
                    0x02 => {self.pulse1_sequence.sequence = 0b00001111; self.pulse1_osc.duty_cycle = 0.500; self.pulse1_osc.duty_idx = 2;},
                    0x03 => {self.pulse1_sequence.sequence = 0b11111100; self.pulse1_osc.duty_cycle = 0.750; self.pulse1_osc.duty_idx = 3;},
                    _    => {}
                }

                self.pulse1_halt        = (data & 0x20) != 0; 
                self.pulse1_env.volume  = (data & 0x0F) as u16;
                self.pulse1_env.disable = (data & 0x10) != 0;
            }, 
            0x4001 => {
                self.pulse1_sweep.enabled = data & 0x80 != 0;
                self.pulse1_sweep.period  = (data & 0x70) >> 4;
                self.pulse1_sweep.down    = data & 0x08 != 0;
                self.pulse1_sweep.shift   = data & 0x07;
                self.pulse1_sweep.reload  = true;
            }, 
            // Control pulse 1 sequencer reload value - first 8 bits
            0x4002 => {
                self.pulse1_sequence.reload = (self.pulse1_sequence.reload & 0xFF00) | data as u16;
            }, 
            // Control pulse 1 sequencer reload value - second 8 bits
            0x4003 => {
                self.pulse1_sequence.reload = (self.pulse1_sequence.reload & 0x00FF) | (((data & 0x07) as u16) << 8);
                self.pulse1_sequence.timer  = self.pulse1_sequence.reload;
                self.pulse1_lc.counter      = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
		        self.pulse1_env.start       = true;
            }, 
            // Set duty cycle of channel 2's pulse wave form
            0x4004 =>  {
                match (data & 0xC0) >> 6 {
                    0x00 => {self.pulse2_sequence.sequence = 0b00000001; self.pulse2_osc.duty_cycle = 0.125; self.pulse2_osc.duty_idx = 0;},
                    0x01 => {self.pulse2_sequence.sequence = 0b00000011; self.pulse2_osc.duty_cycle = 0.250; self.pulse2_osc.duty_idx = 1;},
                    0x02 => {self.pulse2_sequence.sequence = 0b00001111; self.pulse2_osc.duty_cycle = 0.500; self.pulse2_osc.duty_idx = 2;},
                    0x03 => {self.pulse2_sequence.sequence = 0b11111100; self.pulse2_osc.duty_cycle = 0.750; self.pulse2_osc.duty_idx = 3;},
                    _    => {}
                }
                self.pulse2_halt        = (data & 0x20) != 0; 
                self.pulse2_env.volume  = (data & 0x0F) as u16;
                self.pulse2_env.disable = (data & 0x10) != 0;
            }
            0x4005 => {
                self.pulse2_sweep.enabled =  data & 0x80 != 0;
                self.pulse2_sweep.period  = (data & 0x70) >> 4;
                self.pulse2_sweep.down    =  data & 0x08 != 0;
                self.pulse2_sweep.shift   =  data & 0x07;
                self.pulse2_sweep.reload  =  true;

            }, 
            0x4006 => {
                self.pulse2_sequence.reload = (self.pulse2_sequence.reload & 0xFF00) | data as u16;
            }, 
            0x4007 => {
                self.pulse2_sequence.reload = (self.pulse2_sequence.reload & 0x00FF) | (((data & 0x07) as u16) << 8);
                self.pulse2_sequence.timer  = self.pulse2_sequence.reload;
                self.pulse2_lc.counter      = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
		        self.pulse2_env.start       = true;
            }, 
            0x4008 => {}, 
            0x400C => {
                self.noise_env.volume  = (data & 0x0F) as u16;
                self.noise_env.disable = (data & 0x10) != 0;
                self.noise_halt        = (data & 0x20) != 0;
            }, 
            0x400E => {
                self.noise_sequence.mode = data & 0x80 != 0;
                match data & 0x0F {
                    0x00 => {self.noise_sequence.reload = 0;   }
                    0x01 => {self.noise_sequence.reload = 4;   }
                    0x02 => {self.noise_sequence.reload = 8;   }
                    0x03 => {self.noise_sequence.reload = 16;  }
                    0x04 => {self.noise_sequence.reload = 32;  }
                    0x05 => {self.noise_sequence.reload = 64;  }
                    0x06 => {self.noise_sequence.reload = 96;  }
                    0x07 => {self.noise_sequence.reload = 128; }
                    0x08 => {self.noise_sequence.reload = 160; }
                    0x09 => {self.noise_sequence.reload = 202; }
                    0x0A => {self.noise_sequence.reload = 254; }
                    0x0B => {self.noise_sequence.reload = 380; }
                    0x0C => {self.noise_sequence.reload = 508; }
                    0x0D => {self.noise_sequence.reload = 1016;}
                    0x0E => {self.noise_sequence.reload = 2034;}
                    0x0F => {self.noise_sequence.reload = 4068;}
                    _    => {}
                }
            }, 
            // Enable and disable pulse 1 sequencer
            0x4015 => {
                self.pulse1_enable = (data & 0x01) != 0;
                self.pulse2_enable = (data & 0x02) != 0;
                self.noise_enable  = (data & 0x04) != 0;
            }, 
            0x400F => {       
                self.pulse1_env.start = true;
                self.pulse2_env.start = true;
                self.noise_env.start  = true;
                self.noise_lc.counter      = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];

            }, 
            _      => {},
        };
    }


}