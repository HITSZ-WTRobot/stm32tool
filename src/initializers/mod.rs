mod cmake;

pub fn init_cmake() -> anyhow::Result<()> {
    cmake::init()
}
