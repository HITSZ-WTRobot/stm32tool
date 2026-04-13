use crate::patches::{Patch, apply_patch};
use tracing::info;

const CMAKE_C_BLOCK: &str = r#"set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS ON)"#;

const CMAKE_C_AND_CXX_BLOCK: &str = r#"set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# disable cxa_atexit to avoid global/static destructor registration
# suitable for bare-metal targets without process exit
add_compile_options(
        $<$<COMPILE_LANGUAGE:CXX>:-fno-use-cxa-atexit>
)"#;

pub fn init() -> anyhow::Result<()> {
    info!("Initializing CMake project...");

    apply_patch(&Patch::Replace {
        file: "CMakeLists.txt".to_string(),
        find: CMAKE_C_BLOCK.to_string(),
        insert: CMAKE_C_AND_CXX_BLOCK.to_string(),
    })?;

    apply_patch(&Patch::Replace {
        file: "CMakeLists.txt".to_string(),
        find: "enable_language(C ASM)".to_string(),
        insert: "enable_language(C CXX ASM)".to_string(),
    })?;

    apply_patch(&Patch::Append {
        file: "CMakeLists.txt".to_string(),
        after: "# Add sources to executable".to_string(),
        insert: r#"file(GLOB_RECURSE USER_CODE_SOURCES "UserCode/*.*")"#.to_string(),
        marker: "USER_CODE_SOURCES".to_string(),
    })?;

    apply_patch(&Patch::Append {
        file: "CMakeLists.txt".to_string(),
        after: "# Add user sources here".to_string(),
        insert: r#"    ${USER_CODE_SOURCES}"#.to_string(),
        marker: r#"${USER_CODE_SOURCES}"#.to_string(),
    })?;

    apply_patch(&Patch::Append {
        file: "CMakeLists.txt".to_string(),
        after: "# Add include paths".to_string(),
        insert: "include_directories(UserCode)".to_string(),
        marker: "include_directories(UserCode)".to_string(),
    })?;

    apply_patch(&Patch::Append {
        file: "CMakeLists.txt".to_string(),
        after: "list(REMOVE_ITEM CMAKE_C_IMPLICIT_LINK_LIBRARIES ob)".to_string(),
        insert: "\ninclude(cmake/wtr_modules.cmake)\
                 \n\
                 \nwtr_link_packages(${CMAKE_PROJECT_NAME})\
                 \n\
                 \n# Add driver module dependencies here\
                 \n# ===================== DEPENDENCIES =====================\
                 \n# e.g.\
                 \n# add_subdirectory(Modules/YourDriver)\
                 \n# target_link_libraries(${CMAKE_PROJECT_NAME} YourDriver)\
                 \n\
                 \n# ======================================================="
            .to_string(),
        marker: "wtr_link_packages(${CMAKE_PROJECT_NAME})".to_string(),
    })?;

    Ok(())
}
