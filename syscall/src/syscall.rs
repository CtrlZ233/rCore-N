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


macro_rules! generate_syscall {
    ($($name:ident($id:ident);)+) => {
        $(
            #[macro_export]
            macro_rules! $name {
                ($a:expr, $b:expr) => {
                    unsafe {
                        $crate::syscall3($crate::SYSCALL_READ, $a, $b.as_mut_ptr() as usize, $b.len());
                    }
                };
                // 异步
                ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr) => {
                    unsafe {
                        $crate::syscall5($a, $b, $c, $d, $e, $f);
                        let async_call = $crate::AsyncCall::new();
                        async_call.await;
                    }
                }
            }
        )+
    };
}

generate_syscall!{
    read(SYSCALL_READ);
}





// // 直接用宏实现系统调用，同步和异步的区别在于参数的不同
// #[macro_export]
// macro_rules! read {
//     // 同步系统调用
//     ($a:expr, $b:expr) => {
//         unsafe {
//             $crate::syscall3($crate::SYSCALL_READ, $a, $b.as_mut_ptr() as usize, $b.len());
//         }
//     };
//     // 异步
//     ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr) => {
//         unsafe {
//             $crate::syscall5($a, $b, $c, $d, $e, $f);
//             let async_call = $crate::AsyncCall::new();
//             async_call.await;
//         }
//     }
// }



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