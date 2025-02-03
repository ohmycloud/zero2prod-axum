use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
