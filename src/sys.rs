use core::ffi::{c_char, c_void};

use ash::vk::TaggedStructure;
use ash::vk::{self, PFN_vkGetDeviceProcAddr};

#[allow(non_camel_case_types)]
pub type PFN_vk_layerGetPhysicalDeviceProcAddr =
    unsafe extern "system" fn(
        instance: vk::Instance,
        p_name: *const c_char,
    ) -> vk::PFN_vkVoidFunction;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NegotiateLayerStructType(pub(crate) i32);
impl NegotiateLayerStructType {
    pub const UNINTIALIZED: Self = Self(0);
    pub const INTERFACE_STRUCT: Self = Self(1);
    #[inline]
    pub const fn from_raw(x: i32) -> Self {
        Self(x)
    }
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NegotiateLayerInterface {
    pub s_type: NegotiateLayerStructType,
    pub p_next: *mut c_void,
    pub loader_layer_interface_version: u32,
    pub pfn_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub pfn_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    pub pfn_get_physical_device_proc_addr: PFN_vk_layerGetPhysicalDeviceProcAddr,
}

#[allow(non_camel_case_types)]
pub type PFN_vkNegotiateLoaderLayerInterfaceVersion =
    unsafe extern "system" fn(p_version_struct: *mut NegotiateLayerInterface) -> vk::Result;
pub const VK_NEGOTIATE_LOADER_LAYER_INTERFACE_VERSION_SYM: &'static str =
    "vkNegotiateLoaderLayerInterfaceVersion";

#[allow(non_camel_case_types)]
pub type PFN_PhysDevExt = unsafe extern "system" fn(phys_device: vk::PhysicalDevice) -> vk::Result;

#[repr(transparent)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerFunction(pub(crate) i32);
impl LayerFunction {
    pub const LAYER_LINK_INFO: Self = Self(0);
    pub const LOADER_DATA_CALLBACK: Self = Self(1);
    pub const LOADER_LAYER_CREATE_DEVICE_CALLBACK: Self = Self(2);
    pub const LOADER_FEATURES: Self = Self(3);
    #[inline]
    pub const fn from_raw(x: i32) -> Self {
        Self(x)
    }
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LayerInstanceLink {
    pub p_next: *mut LayerInstanceLink,
    pub pfn_next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub pfn_next_get_physical_device_proc_addr: PFN_vk_layerGetPhysicalDeviceProcAddr,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LayerDeviceInfo {
    pub p_device_info: *mut c_void,
    pub pfn_next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
}

#[allow(non_camel_case_types)]
pub type PFN_vkSetInstanceLoaderData =
    unsafe extern "system" fn(instance: vk::Instance, p_object: *mut c_void) -> vk::Result;

#[allow(non_camel_case_types)]
pub type PFN_vkSetDeviceLoaderData =
    unsafe extern "system" fn(device: vk::Device, p_object: *mut c_void) -> vk::Result;

#[allow(non_camel_case_types)]
pub type PFN_vkLayerCreateDevice = unsafe extern "system" fn(
    instance: vk::Instance,
    physical_device: vk::PhysicalDevice,
    p_create_info: *const vk::DeviceCreateInfo,
    p_allocator: *const vk::AllocationCallbacks,
    p_device: *mut vk::Device,
    pfn_layer_GIPA: vk::PFN_vkGetInstanceProcAddr,
    p_pfn_next_GDPA: *mut vk::PFN_vkGetDeviceProcAddr,
) -> vk::Result;

#[allow(non_camel_case_types)]
pub type PFN_vkLayerDestroyDevice = unsafe extern "system" fn(
    physical_device: vk::PhysicalDevice,
    p_allocator: *const vk::AllocationCallbacks,
    pfn_destroy_function: vk::PFN_vkDestroyDevice,
) -> ();

#[repr(transparent)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoaderFeatureFlagBits(pub(crate) vk::Flags);
ash::vk_bitflags_wrapped!(LoaderFeatureFlagBits, vk::Flags);
impl LoaderFeatureFlagBits {
    pub const PHYSICAL_DEVICE_SORTING: Self = Self(0b1);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LayerInstanceCreateInfoLayerDevice {
    pub pfn_layer_create_device: PFN_vkLayerCreateDevice,
    pub pfn_layer_destroy_device: PFN_vkLayerDestroyDevice,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LayerInstanceCreateInfoUnion {
    pub p_layer_info: *mut LayerInstanceLink,
    pub pfn_set_instance_loader_data: PFN_vkSetInstanceLoaderData,
    pub layer_device: LayerInstanceCreateInfoLayerDevice,
    pub loader_features: LoaderFeatureFlagBits,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LayerInstanceCreateInfo {
    pub s_type: vk::StructureType,
    pub p_next: *const c_void,
    pub function: LayerFunction,
    pub u: LayerInstanceCreateInfoUnion,
}
unsafe impl TaggedStructure for LayerInstanceCreateInfo {
    const STRUCTURE_TYPE: vk::StructureType = vk::StructureType::LOADER_INSTANCE_CREATE_INFO;
}
impl ::core::default::Default for LayerInstanceCreateInfo {
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ::core::ptr::null(),
            function: LayerFunction::LAYER_LINK_INFO,
            u: LayerInstanceCreateInfoUnion {
                p_layer_info: ::core::ptr::null_mut(),
            },
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LayerDeviceLink {
    pub p_next: *mut LayerDeviceLink,
    pub pfn_next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub pfn_next_get_device_proc_addr: PFN_vkGetDeviceProcAddr,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LayerDeviceCreateInfoUnion {
    pub p_layer_info: *mut LayerDeviceLink,
    pub pfn_set_device_loader_data: PFN_vkSetDeviceLoaderData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LayerDeviceCreateInfo {
    pub s_type: vk::StructureType,
    pub p_next: *const c_void,
    pub function: LayerFunction,
    pub u: LayerDeviceCreateInfoUnion,
}
unsafe impl TaggedStructure for LayerDeviceCreateInfo {
    const STRUCTURE_TYPE: vk::StructureType = vk::StructureType::LOADER_DEVICE_CREATE_INFO;
}
impl ::core::default::Default for LayerDeviceCreateInfo {
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ::core::ptr::null(),
            function: LayerFunction::LAYER_LINK_INFO,
            u: LayerDeviceCreateInfoUnion {
                p_layer_info: ::core::ptr::null_mut(),
            },
        }
    }
}

#[repr(transparent)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainType(pub(crate) i32);
impl ChainType {
    pub const UNKNOWN: Self = Self(0);
    pub const ENUMERATE_INSTANCE_EXTENSION_PROPERTIES: Self = Self(1);
    pub const ENUMERATE_INSTANCE_LAYER_PROPERTIES: Self = Self(2);
    pub const ENUMERATE_INSTANCE_VERSION: Self = Self(3);
    #[inline]
    pub const fn from_raw(x: i32) -> Self {
        Self(x)
    }
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(C)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub struct ChainHeader {
    ty: ChainType,
    version: u32,
    size: u32,
}

#[allow(non_camel_case_types)]
pub type PFN_layer_vkEnumerateInstanceExtensionProperties = unsafe extern "system" fn(
    p_chain: *const EnumerateInstanceExtensionPropertiesChain,
    p_layer_name: *const c_char,
    p_property_count: *mut u32,
    p_properties: *mut vk::ExtensionProperties,
)
    -> vk::Result;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EnumerateInstanceExtensionPropertiesChain {
    pub header: ChainHeader,
    pub pfn_next_layer: PFN_layer_vkEnumerateInstanceExtensionProperties,
    pub p_next_link: *const EnumerateInstanceExtensionPropertiesChain,
}
impl EnumerateInstanceExtensionPropertiesChain {
    #[inline]
    pub unsafe fn call_down(
        &self,
        p_layer_name: *const c_char,
        p_property_count: *mut u32,
        p_properties: *mut vk::ExtensionProperties,
    ) -> vk::Result {
        (self.pfn_next_layer)(
            self.p_next_link,
            p_layer_name,
            p_property_count,
            p_properties,
        )
    }
}

#[allow(non_camel_case_types)]
pub type PFN_layer_vkEnumerateInstanceLayerProperties = unsafe extern "system" fn(
    p_chain: *const EnumerateInstanceLayerPropertiesChain,
    p_property_count: *mut u32,
    p_properties: *mut vk::LayerProperties,
) -> vk::Result;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EnumerateInstanceLayerPropertiesChain {
    pub header: ChainHeader,
    pub pfn_next_layer: PFN_layer_vkEnumerateInstanceLayerProperties,
    pub p_next_link: *const EnumerateInstanceLayerPropertiesChain,
}
impl EnumerateInstanceLayerPropertiesChain {
    #[inline]
    pub unsafe fn call_down(
        &self,
        p_property_count: *mut u32,
        p_properties: *mut vk::LayerProperties,
    ) -> vk::Result {
        (self.pfn_next_layer)(self.p_next_link, p_property_count, p_properties)
    }
}

#[allow(non_camel_case_types)]
pub type PFN_layer_vkEnumerateInstanceVersion = unsafe extern "system" fn(
    p_chain: *const EnumerateInstanceVersionChain,
    p_api_version: *mut u32,
) -> vk::Result;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EnumerateInstanceVersionChain {
    pub header: ChainHeader,
    pub pfn_next_layer: PFN_layer_vkEnumerateInstanceVersion,
    pub p_next_link: *const EnumerateInstanceVersionChain,
}
impl EnumerateInstanceVersionChain {
    #[inline]
    pub unsafe fn call_down(&self, p_api_version: *mut u32) -> vk::Result {
        (self.pfn_next_layer)(self.p_next_link, p_api_version)
    }
}
