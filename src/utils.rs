use crate::*;

use ash::vk;
use ash::vk::TaggedStructure;

use core::ptr::NonNull;

pub unsafe fn get_instance_chain_info(
    create_info: &vk::InstanceCreateInfo,
    function: LayerFunction,
) -> Option<NonNull<LayerInstanceCreateInfo>> {
    let mut chain_info_ptr = create_info.p_next.cast::<LayerInstanceCreateInfo>();
    while !chain_info_ptr.is_null() {
        let chain_info = chain_info_ptr.read();
        if chain_info.s_type == LayerInstanceCreateInfo::STRUCTURE_TYPE
            && chain_info.function == function
        {
            return Some(NonNull::new_unchecked(chain_info_ptr as _));
        }
        chain_info_ptr = chain_info.p_next.cast()
    }
    None
}

pub unsafe fn get_device_chain_info(
    create_info: &vk::DeviceCreateInfo,
    function: LayerFunction,
) -> Option<NonNull<LayerDeviceCreateInfo>> {
    let mut chain_info_ptr = create_info.p_next.cast::<LayerDeviceCreateInfo>();
    while !chain_info_ptr.is_null() {
        let chain_info = chain_info_ptr.read();
        if chain_info.s_type == LayerDeviceCreateInfo::STRUCTURE_TYPE
            && chain_info.function == function
        {
            return Some(NonNull::new_unchecked(chain_info_ptr as _));
        }
        chain_info_ptr = chain_info.p_next.cast()
    }
    None
}
