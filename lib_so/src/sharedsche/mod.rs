// //! 这个库暴露出共享调度器中使用的数据结构以及接口
// //! 将 `Executor` 数据结构暴露出来，避免在内核和 user_lib 中重复定义
// //! 进程需要在自己的地址空间中声明这个对象
// //! 共享调度器通过 `Executor` 对象的虚拟地址来完成对应的操作
// //! 
// //! 暴露的接口会通过单例模式供内核和用户程序使用（内核和用户进程各自都有实例实例）
// //! 这个模块需要手动指出接口的函数指针在 GOT 表中的偏移，因此在 `fun_offset` 中定义了一系列常量
// //! `UnifiScheFunc(usize)` 表示共享调度器的接口实例


mod bitmap;
mod coroutine;
mod executor;

// extern crate alloc;

pub use executor::Executor;
pub use coroutine::{CoroutineId, Coroutine, CoroutineKind};
use bitmap::BitMap;
