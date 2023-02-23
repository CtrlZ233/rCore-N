
/// 手动的指出各个函数地址在共享库接口中的偏移
pub const USER_ENTRY: usize     = 0;
pub const MAX_PRIO_PID: usize       = 1;
pub const ADD_COROUTINE: usize      = 2;
pub const POLL_KERNEL_FUTURE: usize = 3;
pub const RE_BACK: usize            = 4;
pub const CURRENT_CID: usize        = 5;
pub const REPRIO: usize             = 6;
pub const ADD_VIRTUAL_CORE: usize   = 7;
pub const UPDATE_PRIO: usize        = 8;
