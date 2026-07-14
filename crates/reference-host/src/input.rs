use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REVISION: &str = "deterministic-host-input-v1";
const KEY_WORD_COUNT: usize = 4;
const MAX_RECORD_TRANSACTIONS: usize = 4_096;
const MAX_RECORD_TRANSITIONS: usize = 16_384;

type HeldKeys = [u64; KEY_WORD_COUNT];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeMessage {
    Key { key: usize, down: bool },
    FocusLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostedMessage {
    Key { key: u8, down: bool, system: bool },
    FocusLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KeyTransition {
    key: u8,
    down: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InputTransaction {
    raw_message_count: u32,
    invalid_key_count: u32,
    repeated_down_count: u32,
    unmatched_up_count: u32,
    focus_loss_count: u32,
    focus_release_count: u32,
    transitions: Vec<KeyTransition>,
}

impl InputTransaction {
    fn new(raw_message_count: usize) -> Self {
        Self {
            raw_message_count: u32::try_from(raw_message_count).unwrap_or(u32::MAX),
            invalid_key_count: 0,
            repeated_down_count: 0,
            unmatched_up_count: 0,
            focus_loss_count: 0,
            focus_release_count: 0,
            transitions: Vec::new(),
        }
    }
}

struct ActiveRecording {
    initial_held: HeldKeys,
    transactions: Vec<InputTransaction>,
    transition_count: usize,
}

#[derive(Clone)]
struct CompletedRecording {
    initial_held: HeldKeys,
    final_held: HeldKeys,
    transactions: Vec<InputTransaction>,
    stream_sha256: String,
}

impl CompletedRecording {
    fn new(active: ActiveRecording, final_held: HeldKeys) -> Self {
        let stream_sha256 = stream_sha256(&active.initial_held, &active.transactions);
        Self {
            initial_held: active.initial_held,
            final_held,
            transactions: active.transactions,
            stream_sha256,
        }
    }

    fn summary_json(&self) -> Value {
        let counters = TransactionCounters::from_transactions(&self.transactions);
        json!({
            "revision": REVISION,
            "transactionCount": self.transactions.len(),
            "rawMessageCount": counters.raw_messages,
            "transitionCount": counters.transitions,
            "invalidKeyCount": counters.invalid_keys,
            "repeatedDownCount": counters.repeated_downs,
            "unmatchedUpCount": counters.unmatched_ups,
            "focusLossCount": counters.focus_losses,
            "focusReleaseCount": counters.focus_releases,
            "initialHeldKeys": held_key_list(&self.initial_held),
            "finalHeldKeys": held_key_list(&self.final_held),
            "initialHeldStateSha256": held_state_sha256(&self.initial_held),
            "finalHeldStateSha256": held_state_sha256(&self.final_held),
            "streamSha256": self.stream_sha256,
        })
    }
}

#[derive(Default)]
struct TransactionCounters {
    transactions: u64,
    raw_messages: u64,
    transitions: u64,
    invalid_keys: u64,
    repeated_downs: u64,
    unmatched_ups: u64,
    focus_losses: u64,
    focus_releases: u64,
}

impl TransactionCounters {
    fn from_transactions(transactions: &[InputTransaction]) -> Self {
        let mut counters = Self::default();
        for transaction in transactions {
            counters.add(transaction);
        }
        counters
    }

    fn add(&mut self, transaction: &InputTransaction) {
        self.transactions += 1;
        self.raw_messages += u64::from(transaction.raw_message_count);
        self.transitions += transaction.transitions.len() as u64;
        self.invalid_keys += u64::from(transaction.invalid_key_count);
        self.repeated_downs += u64::from(transaction.repeated_down_count);
        self.unmatched_ups += u64::from(transaction.unmatched_up_count);
        self.focus_losses += u64::from(transaction.focus_loss_count);
        self.focus_releases += u64::from(transaction.focus_release_count);
    }
}

pub struct HostInput {
    held: HeldKeys,
    counters: TransactionCounters,
    active_recording: Option<ActiveRecording>,
    completed_recording: Option<CompletedRecording>,
    recording_fault: Option<String>,
    transaction_capacity: usize,
    transition_capacity: usize,
}

impl HostInput {
    pub fn new() -> Self {
        Self::with_limits(MAX_RECORD_TRANSACTIONS, MAX_RECORD_TRANSITIONS)
    }

    fn with_limits(transaction_capacity: usize, transition_capacity: usize) -> Self {
        Self {
            held: [0; KEY_WORD_COUNT],
            counters: TransactionCounters::default(),
            active_recording: None,
            completed_recording: None,
            recording_fault: None,
            transaction_capacity,
            transition_capacity,
        }
    }

    pub fn ingest(&mut self, messages: Vec<NativeMessage>) {
        if messages.is_empty() {
            return;
        }
        let transaction = normalize(&mut self.held, messages);
        self.counters.add(&transaction);

        let Some(recording) = self.active_recording.as_mut() else {
            return;
        };
        let next_transition_count = recording
            .transition_count
            .saturating_add(transaction.transitions.len());
        if recording.transactions.len() >= self.transaction_capacity
            || next_transition_count > self.transition_capacity
        {
            let message = format!(
                "input record exceeded {} transactions or {} transitions",
                self.transaction_capacity, self.transition_capacity
            );
            self.active_recording = None;
            self.recording_fault = Some(message);
            return;
        }
        recording.transition_count = next_transition_count;
        recording.transactions.push(transaction);
    }

    pub fn start_recording(&mut self) -> Result<Value> {
        ensure!(
            self.active_recording.is_none(),
            "input recording is already active"
        );
        self.recording_fault = None;
        self.active_recording = Some(ActiveRecording {
            initial_held: self.held,
            transactions: Vec::new(),
            transition_count: 0,
        });
        Ok(self.status_json())
    }

    pub fn stop_recording(&mut self) -> Result<Value> {
        if let Some(fault) = &self.recording_fault {
            bail!("input recording failed: {fault}");
        }
        let active = self
            .active_recording
            .take()
            .context("input recording is not active")?;
        let completed = CompletedRecording::new(active, self.held);
        let summary = completed.summary_json();
        self.completed_recording = Some(completed);
        Ok(summary)
    }

    pub fn replay(&self) -> Result<Value> {
        ensure!(
            self.active_recording.is_none(),
            "input recording must stop before replay"
        );
        let recording = self
            .completed_recording
            .as_ref()
            .context("no completed input recording is available")?;
        let live_before = self.held;
        let mut replay_held = recording.initial_held;
        for transaction in &recording.transactions {
            for transition in &transaction.transitions {
                let held = key_is_held(&replay_held, transition.key);
                ensure!(
                    held != transition.down,
                    "record contains a non-state-changing key transition"
                );
                set_key(&mut replay_held, transition.key, transition.down);
            }
        }
        let replay_hash = stream_sha256(&recording.initial_held, &recording.transactions);
        ensure!(
            replay_hash == recording.stream_sha256,
            "replayed input stream hash diverged"
        );
        ensure!(
            replay_held == recording.final_held,
            "replayed held-key state diverged"
        );
        ensure!(live_before == self.held, "input replay mutated live state");

        let mut result = recording.summary_json();
        result["matchesRecord"] = json!(true);
        result["liveStateUnchanged"] = json!(true);
        Ok(result)
    }

    pub fn status_json(&self) -> Value {
        let active = self.active_recording.as_ref();
        json!({
            "revision": REVISION,
            "heldKeys": held_key_list(&self.held),
            "heldStateSha256": held_state_sha256(&self.held),
            "rawMessageCount": self.counters.raw_messages,
            "transactionCount": self.counters.transactions,
            "transitionCount": self.counters.transitions,
            "invalidKeyCount": self.counters.invalid_keys,
            "repeatedDownCount": self.counters.repeated_downs,
            "unmatchedUpCount": self.counters.unmatched_ups,
            "focusLossCount": self.counters.focus_losses,
            "focusReleaseCount": self.counters.focus_releases,
            "recording": {
                "active": active.is_some(),
                "fault": self.recording_fault.as_deref(),
                "transactionCount": active.map_or(0, |recording| recording.transactions.len()),
                "transitionCount": active.map_or(0, |recording| recording.transition_count),
                "transactionCapacity": self.transaction_capacity,
                "transitionCapacity": self.transition_capacity,
            },
            "completedRecord": self
                .completed_recording
                .as_ref()
                .map(CompletedRecording::summary_json),
        })
    }

    pub fn is_held(&self, key: u8) -> bool {
        key != 0 && key_is_held(&self.held, key)
    }
}

impl Default for HostInput {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize(held: &mut HeldKeys, messages: Vec<NativeMessage>) -> InputTransaction {
    let mut transaction = InputTransaction::new(messages.len());
    for message in messages {
        match message {
            NativeMessage::Key { key, down } => {
                let Ok(key) = u8::try_from(key) else {
                    transaction.invalid_key_count += 1;
                    continue;
                };
                if key == 0 {
                    transaction.invalid_key_count += 1;
                    continue;
                }
                let was_held = key_is_held(held, key);
                if down == was_held {
                    if down {
                        transaction.repeated_down_count += 1;
                    } else {
                        transaction.unmatched_up_count += 1;
                    }
                    continue;
                }
                set_key(held, key, down);
                transaction.transitions.push(KeyTransition { key, down });
            }
            NativeMessage::FocusLost => {
                transaction.focus_loss_count += 1;
                for key in 1..=u8::MAX {
                    if key_is_held(held, key) {
                        set_key(held, key, false);
                        transaction
                            .transitions
                            .push(KeyTransition { key, down: false });
                        transaction.focus_release_count += 1;
                    }
                }
            }
        }
    }
    transaction
}

fn key_is_held(held: &HeldKeys, key: u8) -> bool {
    let index = usize::from(key);
    held[index / 64] & (1_u64 << (index % 64)) != 0
}

fn set_key(held: &mut HeldKeys, key: u8, down: bool) {
    let index = usize::from(key);
    let mask = 1_u64 << (index % 64);
    if down {
        held[index / 64] |= mask;
    } else {
        held[index / 64] &= !mask;
    }
}

fn held_key_list(held: &HeldKeys) -> Vec<u8> {
    (1..=u8::MAX)
        .filter(|key| key_is_held(held, *key))
        .collect()
}

fn held_state_sha256(held: &HeldKeys) -> String {
    let mut digest = Sha256::new();
    digest.update(b"wulin-host-input-held-v1\0");
    for word in held {
        digest.update(word.to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn stream_sha256(initial_held: &HeldKeys, transactions: &[InputTransaction]) -> String {
    let mut digest = Sha256::new();
    digest.update(b"wulin-host-input-stream-v1\0");
    for word in initial_held {
        digest.update(word.to_le_bytes());
    }
    digest.update((transactions.len() as u64).to_le_bytes());
    for (index, transaction) in transactions.iter().enumerate() {
        digest.update((index as u64).to_le_bytes());
        digest.update(transaction.raw_message_count.to_le_bytes());
        digest.update(transaction.invalid_key_count.to_le_bytes());
        digest.update(transaction.repeated_down_count.to_le_bytes());
        digest.update(transaction.unmatched_up_count.to_le_bytes());
        digest.update(transaction.focus_loss_count.to_le_bytes());
        digest.update(transaction.focus_release_count.to_le_bytes());
        digest.update((transaction.transitions.len() as u32).to_le_bytes());
        for transition in &transaction.transitions {
            digest.update([transition.key, u8::from(transition.down)]);
        }
    }
    format!("{:x}", digest.finalize())
}

#[cfg(test)]
#[path = "../tests/private/input.rs"]
mod tests;
