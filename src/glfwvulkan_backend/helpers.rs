use ash::vk;
use colored::*;
use std::ffi::CStr;

/// Standardized info logging: [AurenFox][Module] Message
pub fn log_info(module: &str, msg: &str) {
    println!(
        "{}{} -> {}{}{} {}",
        "Auren".bright_cyan().bold(),
        "Fox".bright_red(),
        "[".white(),
        module.bright_yellow(),
        "]".white(),
        msg
    );
}

/// Standardized warning logging
pub fn log_warn(module: &str, msg: &str) {
    println!(
        "{}{} -> {}{}{} {}",
        "Auren".bright_cyan().bold(),
        "Fox".bright_red(),
        "[".white(),
        module.bright_red(),
        "]".white(),
        msg.bright_yellow()
    );
}

/// Standardized error logging (Panics)
pub fn log_err(module: &str, msg: &str) -> ! {
    panic!(
        "{}{} -> {}{}{} {}",
        "Auren".bright_cyan().bold(),
        "Fox".bright_red(),
        "[".white(),
        module.on_red().white(),
        "]".white(),
        msg.bright_red().bold()
    );
}

/// Required for Buffer/Image creation: Finds the memory index on your specific GPU
/// that matches the requirements (e.g., Host Visible for CPUs to write to)
pub fn find_memory_type(
    mem_properties: &vk::PhysicalDeviceMemoryProperties,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
        {
            return i;
        }
    }
    log_err("Memory", "Failed to find suitable memory type on GPU!");
}

/// Converts Vulkan C-strings (like extension names) to Rust Strings safely
pub unsafe fn vk_to_string(raw_string: *const std::os::raw::c_char) -> String {
    if raw_string.is_null() {
        return "Unknown".to_string();
    }
    unsafe { CStr::from_ptr(raw_string).to_string_lossy().into_owned() }
}