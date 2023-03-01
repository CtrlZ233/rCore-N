use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::{sync::Arc, task::Wake};
use core::task::{Waker, Poll, Context};
use spin::Mutex;

/// 协程 Id
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct CoroutineId(pub usize);

impl CoroutineId {
    /// 生成新的协程 Id
    pub fn generate() -> CoroutineId {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > usize::MAX / 2 {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        CoroutineId(id)
    }
    /// 根据 usize 生成协程 Id
    pub fn from_val(v: usize) -> Self {
        Self(v)
    }
    /// 获取协程 Id 的 usize
    pub fn get_val(&self) -> usize {
        self.0
    } 
}

/// 协程 waker，在这里只提供一个上下文
struct CoroutineWaker(CoroutineId);

impl CoroutineWaker {
    /// 新建协程 waker
    pub fn new(cid: CoroutineId) -> Waker {
        Waker::from(Arc::new(Self(cid)))
    }
}

impl Wake for CoroutineWaker {
    fn wake(self: Arc<Self>) { }
    fn wake_by_ref(self: &Arc<Self>) { }
}


/// 协程，包装了 future，优先级，以及提供上下文的 waker，内核来唤醒或者内核、外部设备发中断，在中断处理程序里面唤醒
pub struct Coroutine{
    /// 协程编号
    pub cid: CoroutineId,
    /// 协程类型
    pub kind: CoroutineKind,
    /// future
    pub inner: Mutex<CoroutineInner>,
}
/// 协程类型
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CoroutineKind {
    /// 内核调度协程
    KernSche,
    /// 内核系统调用协程
    KernSyscall,
    /// 用户协程
    UserNorm,
}

pub struct CoroutineInner {
    pub future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, 
    /// 当前协程的优先级
    pub prio: usize,
    /// waker
    pub waker: Arc<Waker>,
}

impl Coroutine {
    /// 生成协程
    pub fn new(future: Pin<Box<dyn Future<Output=()> + Send + Sync>>, prio: usize, kind: CoroutineKind) -> Arc<Self> {
        let cid = CoroutineId::generate();
        Arc::new(
            Coroutine {
                cid,
                kind,
                inner: Mutex::new(CoroutineInner {
                    future,
                    prio,
                    waker: Arc::new(CoroutineWaker::new(cid)),
                })
                
            }
        )
    }
    /// 执行
    pub fn execute(self: Arc<Self>) -> Poll<()> {
        let mut inner = self.inner.lock();
        let waker = inner.waker.clone();
        let mut context = Context::from_waker(&*waker);
        inner.future.as_mut().poll(&mut context)
    }
}
