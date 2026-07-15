use reference_host::HostElapsedSample;

pub(crate) const fn admitted_elapsed(sample: HostElapsedSample) -> Option<u64> {
    match sample {
        HostElapsedSample::Ready {
            elapsed_nanoseconds,
        } => Some(elapsed_nanoseconds),
        HostElapsedSample::Reset
        | HostElapsedSample::Stalled { .. }
        | HostElapsedSample::Suspended => None,
    }
}
