const KEY_WORD_COUNT: usize = 4;

type KeyBits = [u64; KEY_WORD_COUNT];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeMessage {
    Key { key: usize, down: bool },
    FocusLost,
}

pub struct HostInput {
    held: KeyBits,
    pressed: KeyBits,
    released: KeyBits,
}

impl HostInput {
    pub fn new() -> Self {
        Self {
            held: [0; KEY_WORD_COUNT],
            pressed: [0; KEY_WORD_COUNT],
            released: [0; KEY_WORD_COUNT],
        }
    }

    pub fn ingest(&mut self, messages: Vec<NativeMessage>) {
        self.pressed = [0; KEY_WORD_COUNT];
        self.released = [0; KEY_WORD_COUNT];
        for message in messages {
            match message {
                NativeMessage::Key { key, down } => {
                    let Ok(key) = u8::try_from(key) else {
                        continue;
                    };
                    if key == 0 || down == key_is_set(&self.held, key) {
                        continue;
                    }
                    set_key(&mut self.held, key, down);
                    let edges = if down {
                        &mut self.pressed
                    } else {
                        &mut self.released
                    };
                    set_key(edges, key, true);
                }
                NativeMessage::FocusLost => {
                    for key in 1..=u8::MAX {
                        if key_is_set(&self.held, key) {
                            set_key(&mut self.held, key, false);
                            set_key(&mut self.released, key, true);
                        }
                    }
                }
            }
        }
    }

    pub fn is_held(&self, key: u8) -> bool {
        key != 0 && key_is_set(&self.held, key)
    }

    pub fn was_pressed(&self, key: u8) -> bool {
        key != 0 && key_is_set(&self.pressed, key)
    }

    pub fn was_released(&self, key: u8) -> bool {
        key != 0 && key_is_set(&self.released, key)
    }
}

impl Default for HostInput {
    fn default() -> Self {
        Self::new()
    }
}

fn key_is_set(keys: &KeyBits, key: u8) -> bool {
    let index = usize::from(key);
    keys[index / 64] & (1_u64 << (index % 64)) != 0
}

fn set_key(keys: &mut KeyBits, key: u8, value: bool) {
    let index = usize::from(key);
    let mask = 1_u64 << (index % 64);
    if value {
        keys[index / 64] |= mask;
    } else {
        keys[index / 64] &= !mask;
    }
}

#[cfg(test)]
#[path = "../tests/private/input.rs"]
mod tests;
