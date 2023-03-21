use core::{future::Future, pin::Pin, task::{Context, Poll}};

// 异步系统调用辅助 future
pub struct AsyncCall {
    blocked: bool,         
}

impl AsyncCall {
    pub fn new() -> Self {
        Self {
            blocked: true
        }
    }
}

impl Future for AsyncCall {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // submit async task to kernel and return immediately
        if self.blocked {
            self.blocked = false;
            return Poll::Pending;
        }
        return Poll::Ready(());
    }
}