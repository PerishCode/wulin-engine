use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostActivation {
    Suspended,
    Resumed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ActivationState {
    current_focused: bool,
    delivered_focused: bool,
    changed: bool,
    initialized: bool,
}

impl ActivationState {
    pub(crate) const fn new() -> Self {
        Self {
            current_focused: false,
            delivered_focused: false,
            changed: false,
            initialized: false,
        }
    }

    pub(crate) fn record(&mut self, focused: bool) {
        if self.current_focused == focused {
            return;
        }
        self.current_focused = focused;
        self.changed = true;
    }

    pub(crate) fn drain(&mut self) -> Vec<HostActivation> {
        if !self.initialized {
            self.initialized = true;
            self.delivered_focused = self.current_focused;
            self.changed = false;
            return vec![activation(self.current_focused)];
        }
        if !self.changed {
            return Vec::new();
        }

        let transitions = match (self.delivered_focused, self.current_focused) {
            (false, true) => vec![HostActivation::Resumed],
            (true, false) => vec![HostActivation::Suspended],
            (false, false) => vec![HostActivation::Resumed, HostActivation::Suspended],
            (true, true) => vec![HostActivation::Suspended, HostActivation::Resumed],
        };
        self.delivered_focused = self.current_focused;
        self.changed = false;
        transitions
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }
}

fn activation(focused: bool) -> HostActivation {
    if focused {
        HostActivation::Resumed
    } else {
        HostActivation::Suspended
    }
}

#[cfg(test)]
#[path = "../tests/private/activation.rs"]
mod tests;
