use crate::initializers::{FPUType, IdeInitArgs, IdeInitializer};
use crate::patches::{apply_patch, Patch};
use tracing::info;

pub struct CMake;

impl IdeInitializer for CMake {
    fn name(&self) -> &'static str {
        "CMake (toolchain: CMake) Compatible with CLion and VSCode (official ST plugin)"
    }

    fn init(&self, args: &IdeInitArgs, _force: bool) -> anyhow::Result<()> {
        info!("Initializing CMake project...");

        if args.fpu == FPUType::Hard {
            apply_patch(&Patch::Append {
                file: "CMakeLists.txt".to_string(),
                after: "# Setup compiler settings\n\
                    set(CMAKE_C_STANDARD 11)\n\
                    set(CMAKE_C_STANDARD_REQUIRED ON)\n\
                    set(CMAKE_C_EXTENSIONS ON)"
                    .to_string(),
                insert: "\n#Uncomment for hardware floating point\n\
                     add_compile_definitions(ARM_MATH_CM4;ARM_MATH_MATRIX_CHECK;ARM_MATH_ROUNDING)\n\
                     add_compile_options(-mfloat-abi=hard -mfpu=fpv4-sp-d16)\n\
                     add_link_options(-mfloat-abi=hard -mfpu=fpv4-sp-d16)\n\n\
                     add_compile_options(-ffunction-sections -fdata-sections -fno-common -fmessage-length=0)\n"
                    .to_string(),
                marker: "#Uncomment for hardware floating point".to_string(),
            })?;
        } else {
            apply_patch(&Patch::Append {
                file: "CMakeLists.txt".to_string(),
                after: "# Setup compiler settings\
                    \nset(CMAKE_C_STANDARD 11)\
                    \nset(CMAKE_C_STANDARD_REQUIRED ON)\
                    \nset(CMAKE_C_EXTENSIONS ON)"
                    .to_string(),
                insert: "\n#Uncomment for software floating point\
                     \nadd_compile_options(-mfloat-abi=soft)\
                     \n\
                     \nadd_compile_options(-ffunction-sections -fdata-sections -fno-common -fmessage-length=0)\n"
                    .to_string(),
                marker: "#Uncomment for hardware floating point".to_string(),
            })?;
        }

        apply_patch(&Patch::Append {
            file: "CMakeLists.txt".to_string(),
            after: "# Add sources to executable".to_string(),
            insert: r#"file(GLOB_RECURSE SOURCES "UserCode/*.*")"#.to_string(),
            marker: r#"file(GLOB_RECURSE SOURCES "UserCode/*.*")"#.to_string(),
        })?;

        apply_patch(&Patch::Append {
            file: "CMakeLists.txt".to_string(),
            after: "# Add user sources here".to_string(),
            insert: r#"    ${SOURCES}"#.to_string(),
            marker: r#"${SOURCES}"#.to_string(),
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
            insert: "\n# Add dependence from library\
                     \n# ===================== DEPENDENCIES =====================
                     \n# e.g.\
                     \n#add_subdirectory(library/motor_drivers/UserCode)\
                     \n\
                     \nset(USER_LIBRARIES \"\")\
                     \n\
                     \n# =======================================================\
                     \n\
                     \n# every library will depend on stm32cubemx\
                     \nforeach (LIBRARY IN LISTS USER_LIBRARIES)\
                     \n    target_link_libraries(${LIBRARY} PRIVATE stm32cubemx)\
                     \nendforeach ()"
                .to_string(),
            marker: "# Add dependence from library".to_string(),
        })?;

        apply_patch(&Patch::Append {
            file: "CMakeLists.txt".to_string(),
            after: "# Add user defined libraries".to_string(),
            insert: "    ${USER_LIBRARIES}".to_string(),
            marker: "${USER_LIBRARIES}".to_string(),
        })?;

        Ok(())
    }
}
