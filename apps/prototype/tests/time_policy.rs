#[path = "../src/time.rs"]
mod time;

use reference_host::HostElapsedSample;

#[test]
fn only_ready_is_admitted() {
    assert_eq!(time::admitted_elapsed(HostElapsedSample::Reset), None);
    assert_eq!(time::admitted_elapsed(HostElapsedSample::Suspended), None);
    assert_eq!(
        time::admitted_elapsed(HostElapsedSample::Stalled {
            elapsed_nanoseconds: 125_000_001,
            maximum_elapsed_nanoseconds: 125_000_000,
        }),
        None
    );
    assert_eq!(
        time::admitted_elapsed(HostElapsedSample::Ready {
            elapsed_nanoseconds: 0,
        }),
        Some(0)
    );
    assert_eq!(
        time::admitted_elapsed(HostElapsedSample::Ready {
            elapsed_nanoseconds: 125_000_000,
        }),
        Some(125_000_000)
    );
}
