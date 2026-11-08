use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

static DELIVERIES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn deliveries() -> &'static Mutex<VecDeque<String>> {
    DELIVERIES.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn claim(key: &str) -> bool {
    let mut entries = deliveries()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if entries.iter().any(|entry| entry == key) {
        return false;
    }
    entries.push_back(key.to_owned());
    // ponytail: retain 4096 identities per process; persist the ledger if older replay matters.
    if entries.len() > 4096 {
        entries.pop_front();
    }
    true
}

pub fn release(key: &str) {
    deliveries()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .retain(|entry| entry != key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guests_share_claims_and_failed_deliveries_can_retry() {
        let key = "session-a:turn:42";
        assert!(claim(key));
        assert!(
            !std::thread::spawn(move || claim(key))
                .join()
                .expect("claim thread panicked")
        );
        assert!(claim("session-a:turn:43"));
        release(key);
        assert!(claim(key));
        release(key);
        release("session-a:turn:43");
    }
}
