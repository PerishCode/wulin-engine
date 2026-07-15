use reference_host::HostInput;

const A: u8 = 0x41;
const D: u8 = 0x44;
const S: u8 = 0x53;
const W: u8 = 0x57;

pub(crate) const CARDINAL_DELTA_Q9: i32 = 32;
pub(crate) const DIAGONAL_COMPONENT_Q9: i32 = 23;
pub(crate) const STEP_UP_LIMIT_Q16: i32 = 32_768;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Command {
    pub(crate) delta_x_q9: i32,
    pub(crate) delta_z_q9: i32,
    pub(crate) step_up_limit_q16: i32,
}

pub(crate) fn command(input: &HostInput) -> Command {
    let x = i32::from(input.is_held(D)) - i32::from(input.is_held(A));
    let z = i32::from(input.is_held(S)) - i32::from(input.is_held(W));
    let component = if x != 0 && z != 0 {
        DIAGONAL_COMPONENT_Q9
    } else {
        CARDINAL_DELTA_Q9
    };
    Command {
        delta_x_q9: x * component,
        delta_z_q9: z * component,
        step_up_limit_q16: STEP_UP_LIMIT_Q16,
    }
}
