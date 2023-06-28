use super::notification::Notification;

pub trait Notifier : Clone + Send {
    fn notify(&self, notification: Notification);
}