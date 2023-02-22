use core::{future::Future, pin::Pin, task::{Context, Poll}};


macro_rules! generate_syscall {
    ($($name:ident | $async_name:ident;)+) => {
        $(
            #[macro_export]
            macro_rules! $name {
                // 同步
                ($a:expr, $b:expr) => {
                    $crate::read($a, $b)
                };
                // 异步
                ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => {
                    $crate::$async_name($a, $b, $c, $d, $e);
                    let async_call = $crate::AsyncCall::new();
                    async_call.await;
                }
            }
        )+
    };
}

generate_syscall!{
    read | async_read;
}

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