/// Shared mutex to serialize tests that mutate the HOME environment variable.
/// All storage test modules must acquire this lock before calling set_var("HOME", ...).
use std::sync::Mutex;

pub static HOME_LOCK: Mutex<()> = Mutex::new(());
