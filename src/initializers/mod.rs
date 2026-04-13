mod cmake;

pub fn init_cmake(enable_non_intrusive_headers: bool) -> anyhow::Result<()> {
    cmake::init(enable_non_intrusive_headers)
}
