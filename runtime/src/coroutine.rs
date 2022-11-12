use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::{
    sync::Arc,
    task::Wake,
};
use core::task::{Waker, Poll, Context};
use spin::Mutex;




#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct CoroutineId(pub usize);

impl CoroutineId {
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

    pub fn get_tid_by_usize(v: usize) -> Self {
        Self(v)
    }

    pub fn get_val(&self) -> usize {
        self.0
    } 
}

struct CoroutineWaker(CoroutineId);

impl CoroutineWaker {
    pub fn new(cid: CoroutineId) -> Waker {
        Waker::from(Arc::new(Self(cid)))
    }
}

impl Wake for CoroutineWaker {
    fn wake(self: Arc<Self>) { }
    fn wake_by_ref(self: &Arc<Self>) { }
}


// 协程，包装了 future，优先级，以及提供上下文的 waker，内核来唤醒或者内核、外部设备发中断，在中断处理程序里面唤醒
pub struct Coroutine{
    // 任务编号
    pub cid: CoroutineId,
    // future
    pub future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, 
    pub prio: usize,
    pub waker: Arc<Waker>,
}

impl Coroutine {
    pub fn new(future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, prio: usize) -> Arc<Self> {
        let cid = CoroutineId::generate();
        Arc::new(
            Coroutine {
                cid,
                future,
                prio,
                waker: Arc::new(CoroutineWaker::new(cid)),
            }
        )
    }

    pub fn execute(self: Arc<Self>) -> Poll<()> {
        let waker = self.waker.clone();
        let mut context = Context::from_waker(&*waker);
        self.future.lock().as_mut().poll(&mut context)
    }
}
