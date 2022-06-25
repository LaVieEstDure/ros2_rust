use crate::rcl_bindings::*;
use crate::{Publisher, RclrsError, ToResult};

use rosidl_runtime_rs::RmwMessage;

use std::ops::{Deref, DerefMut};

/// A message that is owned by the middleware, loaned for publishing.
///
/// It dereferences to a `&mut T`.
///
/// This type is returned by [`Publisher::borrow_loaned_message()`], see the documentation of
/// that function for more information.
///
/// The loan is returned by dropping the message or [publishing it][1].
///
/// [1]: LoanedMessage::publish
pub struct LoanedMessage<'a, T>
where
    T: RmwMessage,
{
    pub(super) msg_ptr: *mut T,
    pub(super) publisher: &'a Publisher<T>,
}

impl<'a, T> Deref for LoanedMessage<'a, T>
where
    T: RmwMessage,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: msg_ptr is a valid pointer, obtained through rcl_borrow_loaned_message.
        unsafe { &*self.msg_ptr }
    }
}

impl<'a, T> DerefMut for LoanedMessage<'a, T>
where
    T: RmwMessage,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: msg_ptr is a valid pointer, obtained through rcl_borrow_loaned_message.
        unsafe { &mut *self.msg_ptr }
    }
}

impl<'a, T> Drop for LoanedMessage<'a, T>
where
    T: RmwMessage,
{
    fn drop(&mut self) {
        // Check whether the loan was already returned with
        // rcl_publish_loaned_message()
        if !self.msg_ptr.is_null() {
            unsafe {
                // SAFETY: These two pointers are valid, and the msg_ptr is not used afterwards.
                rcl_return_loaned_message_from_publisher(
                    &*self.publisher.rcl_publisher_mtx.lock(),
                    self.msg_ptr as *mut _,
                )
                .ok()
                .unwrap()
            }
        }
    }
}

impl<'a, T> LoanedMessage<'a, T>
where
    T: RmwMessage,
{
    /// Publishes the loaned message, falling back to regular publishing if needed.
    pub fn publish(mut self) -> Result<(), RclrsError> {
        unsafe {
            // SAFETY: These two pointers are valid, and the msg_ptr is not used afterwards.
            rcl_publish_loaned_message(
                &*self.publisher.rcl_publisher_mtx.lock(),
                self.msg_ptr as *mut _,
                std::ptr::null_mut(),
            )
            .ok()?;
        }
        // Set the msg_ptr to null, as a signal to the drop impl that this
        // loan was already returned.
        self.msg_ptr = std::ptr::null_mut();
        Ok(())
    }
}
