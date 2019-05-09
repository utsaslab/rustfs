pub const BLOCK_SIZE: usize = 512;
pub const LIST_SIZE: usize = 16;
pub const INODE_SIZE: usize = 32;
pub const DIR_TYPE: u64 = 2;
pub const FILE_TYPE: u64 = 1;
pub const O_RDONLY: u32 = (1 << 0);
pub const O_WRONLY: u32 = (1 << 1);
pub const O_RDWR: u32 = (1 << 2);
pub const O_NONBLOCK: u32 = (1 << 3);
pub const O_APPEND: u32 = (1 << 4);
pub const O_CREAT: u32 = (1 << 5);
pub const DEFAULT_SERVER1_SOCKET_PATH: &str = "/tmp/rustfs_server1.sock";
pub const DEFAULT_SERVER2_SOCKET_PATH: &str = "/tmp/rustfs_server2.sock";

/// Support FS operations
pub const FS_SHUTDOWN: &str = "shutdown";
pub const FS_OPEN: &str = "open";
