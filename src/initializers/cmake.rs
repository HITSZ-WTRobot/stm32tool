use crate::patches::{Patch, apply_patch};
use tracing::info;

pub fn init(enable_non_intrusive_headers: bool) -> anyhow::Result<()> {
    info!("Initializing CMake project...");

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

    if enable_non_intrusive_headers {
        info!("Generating CMake non-intrusive header configuration");
        apply_patch(&Patch::Append {
            file: "CMakeLists.txt".to_string(),
            after: "# Add include paths".to_string(),
            insert: "\n# 非侵入式引入头文件\ntarget_compile_options(${CMAKE_PROJECT_NAME} PRIVATE -include ${CMAKE_SOURCE_DIR}/UserCode/app/app.h)\n".to_string(),
            marker: "UserCode/app/app.h".to_string(),
        })?;
    }

    apply_patch(&Patch::Append {
        file: "CMakeLists.txt".to_string(),
        after: "list(REMOVE_ITEM CMAKE_C_IMPLICIT_LINK_LIBRARIES ob)".to_string(),
        insert: "\n# Add driver module dependencies here\
                 \n# ===================== DEPENDENCIES =====================\
                 \n# e.g.\
                 \n# add_subdirectory(Modules/YourDriver)\
                 \n# target_link_libraries(${CMAKE_PROJECT_NAME} YourDriver)\
                 \n\
                 \n# ======================================================="
            .to_string(),
        marker: "# Add driver module dependencies here".to_string(),
    })?;

    Ok(())
}
