use core::{future::Future, pin::Pin, task::{Context, Poll}};



macro_rules! syscall {
    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, $($g:ident)?)?)?)?)?)?);)+) => {
        $(
            pub unsafe fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize, $($g: usize)?)?)?)?)?)?) -> isize {
                let ret: isize;
                core::arch::asm!(
                    "ecall",
                    in("a7") $a,
                    $(
                        in("a0") $b,
                        $(
                            in("a1") $c,
                            $(
                                in("a2") $d,
                                $(
                                    in("a3") $e,
                                    $(
                                        in("a4") $f,
                                        $(
                                            in("a5") $g,
                                        )?
                                    )?
                                )?
                            )?
                        )?
                    )?
                    lateout("a0") ret,
                    options(nostack),
                );
                ret
            }
        )+
    };
}

syscall! {
    syscall0(a,);
    syscall1(a, b,);
    syscall2(a, b, c,);
    syscall3(a, b, c, d,);
    syscall4(a, b, c, d, e,);
    syscall5(a, b, c, d, e, f,);
    syscall6(a, b, c, d, e, f, g);
}

macro_rules! asyscall {
    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, $($g:ident)?)?)?)?)?)?);)+) => {
        $(
            pub async fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize, $($g: usize)?)?)?)?)?)?) {
                let ret: isize;
                unsafe {
                    core::arch::asm!(
                        "ecall",
                        in("a7") $a,
                        $(
                            in("a0") $b,
                            $(
                                in("a1") $c,
                                $(
                                    in("a2") $d,
                                    $(
                                        in("a3") $e,
                                        $(
                                            in("a4") $f,
                                            $(
                                                in("a5") $g,
                                            )?
                                        )?
                                    )?
                                )?
                            )?
                        )?
                        lateout("a0") ret,
                        options(nostack),
                    );
                }
                let async_call = AsyncCall::new();
                async_call.await;
            }
        )+
    };
}

/// 这里定义的函数是暴露出来给 user_app 使用，因为 async fn 和 普通的 fn 不能同名，因此还需要思考一下其他方法
asyscall! {
    async_read(call_type, fd, buffer_ptr, buffer_len, key, cid,);
}

pub const ASYNC_SYSCALL_READ: usize = 2501;
pub const ASYNC_SYSCALL_WRITE: usize = 2502;


// 异步系统调用, 顶层 future
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