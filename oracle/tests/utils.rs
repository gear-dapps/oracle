use gtest::{Program, System};

pub const OWNER: u64 = 3;
pub const MANAGER: u64 = 4;
pub const NEW_MANAGER: u64 = 5;
pub const USER: u64 = 6;
pub const FAKE_OWNER: u64 = 7;
pub const FAKE_MANAGER: u64 = 8;

pub fn load_program(sys: &System) -> Program<'_> {
    sys.init_logger();

    Program::current(sys)
}
