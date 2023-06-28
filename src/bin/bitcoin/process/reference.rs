use super::error_process::ErrorProcess;

use std::sync::{Arc, Mutex, MutexGuard};

pub type MutArc<T> = Arc<Mutex<T>>;

/// Get the value of a mutable reference given by Arc<Mutex<T>>
///
/// ### Error
///  * `ErrorUI::CannotGetInner`: It will appear when we try to get the inner value of a mutex
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
pub fn get_inner<T>(reference: MutArc<T>) -> Result<T, ErrorProcess> {
    match Arc::try_unwrap(reference) {
        Ok(reference_unwrap) => match reference_unwrap.into_inner() {
            Ok(reference) => Ok(reference),
            Err(_) => Err(ErrorProcess::CannotGetInner),
        },
        Err(_) => Err(ErrorProcess::CannotUnwrapArc),
    }
}

/// Get a mutable guard to use the value inside the Arc<Mutex<T>>
///
/// ### Error
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
pub fn get_reference<T>(reference: &MutArc<T>) -> Result<MutexGuard<'_, T>, ErrorProcess> {
    match reference.lock() {
        Ok(reference) => Ok(reference),
        Err(_) => Err(ErrorProcess::CannotUnwrapArc),
    }
}
