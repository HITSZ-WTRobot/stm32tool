mod cmake;

pub const CXX_STANDARD_WARNING: &str =
    "stm32tool defaults to C++20; ensure your compiler and dependencies support C++20.";

pub fn init_cmake() -> anyhow::Result<()> {
    cmake::init()
}
