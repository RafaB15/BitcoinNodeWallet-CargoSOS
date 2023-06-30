use super::notification::Notification;

/// This trait is used to notify the user of the different events that occur
/// throughout the execution of the program.
pub trait Notifier: Clone + Send {
    fn notify(&self, notification: Notification);
}
