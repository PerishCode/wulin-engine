use reference_host::HostInput;

const CLOCKWISE: u8 = 0x45;
const COUNTER_CLOCKWISE: u8 = 0x51;
const ORBIT_COUNT: i8 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rig {
    pub(crate) orbit_index: u8,
    pub(crate) position_offset: [f32; 3],
    pub(crate) target_offset: [f32; 3],
    pub(crate) vertical_fov_degrees: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Candidate(Rig);

impl Candidate {
    pub(crate) const fn rig(self) -> Rig {
        self.0
    }
}

pub(crate) struct Policy {
    committed_orbit_index: u8,
}

impl Policy {
    pub(crate) const fn new() -> Self {
        Self {
            committed_orbit_index: 0,
        }
    }

    pub(crate) fn candidate(&self, input: &HostInput) -> Candidate {
        let delta =
            i8::from(input.was_pressed(CLOCKWISE)) - i8::from(input.was_pressed(COUNTER_CLOCKWISE));
        let orbit_index = (self.committed_orbit_index as i8 + delta).rem_euclid(ORBIT_COUNT) as u8;
        Candidate(rig(orbit_index))
    }

    pub(crate) fn commit(&mut self, candidate: Candidate) {
        self.committed_orbit_index = candidate.0.orbit_index;
    }
}

const fn rig(orbit_index: u8) -> Rig {
    let (position_offset, target_offset) = match orbit_index {
        0 => ([9.0, 4.0, 12.0], [0.0, -1.0, -3.0]),
        1 => ([12.0, 4.0, -9.0], [-3.0, -1.0, 0.0]),
        2 => ([-9.0, 4.0, -12.0], [0.0, -1.0, 3.0]),
        3 => ([-12.0, 4.0, 9.0], [3.0, -1.0, 0.0]),
        _ => unreachable!(),
    };
    Rig {
        orbit_index,
        position_offset,
        target_offset,
        vertical_fov_degrees: 60.0,
    }
}
