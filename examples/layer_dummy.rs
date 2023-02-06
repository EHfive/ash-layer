use core::ffi::{c_char, CStr};
use core::mem;

use ash::extensions::khr;
use ash::vk;
use ash_layer::*;
use dashmap::DashMap;
use once_cell::sync::{Lazy, OnceCell};

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

macro_rules! log {
    ($($arg:tt)+) =>
    (eprintln!("[Ash Layer Dummy][{}] {}", function!(), format_args!($($arg)*)));
}

#[allow(dead_code)]
struct LayerInstance {
    ash_instance: ash::Instance,
    khr_surface: khr::Surface,
}

#[allow(dead_code)]
struct LayerDevice {
    instance: vk::Instance,
    ash_device: ash::Device,
    khr_swapchain: khr::Swapchain,
}

static GIPA: OnceCell<vk::PFN_vkGetInstanceProcAddr> = OnceCell::new();
static GPHYPA: OnceCell<PFN_vk_layerGetPhysicalDeviceProcAddr> = OnceCell::new();
static ENTRY: OnceCell<ash::Entry> = OnceCell::new();

// DashMap ensures thread-safely and can be consider as faster mutex guarded HashMap
static INSTANCE_MAP: Lazy<DashMap<vk::Instance, LayerInstance>> = Lazy::new(|| DashMap::new());
static PHY_TO_INSTANCE_MAP: Lazy<DashMap<vk::PhysicalDevice, vk::Instance>> =
    Lazy::new(|| DashMap::new());
static GDPA_MAP: Lazy<DashMap<vk::Device, vk::PFN_vkGetDeviceProcAddr>> =
    Lazy::new(|| DashMap::new());
static DEVICE_MAP: Lazy<DashMap<vk::Device, LayerDevice>> = Lazy::new(|| DashMap::new());

#[no_mangle]
#[doc = "https://vulkan.lunarg.com/doc/view/1.3.236.0/linux/LoaderLayerInterface.html#user-content-layer-interface-version-2"]
pub unsafe extern "system" fn vkNegotiateLoaderLayerInterfaceVersion(
    p_version_struct: *mut NegotiateLayerInterface,
) -> vk::Result {
    let version_struct = &mut *p_version_struct;
    log!(
        "loader LayerInterfaceVersion: {}",
        version_struct.loader_layer_interface_version,
    );
    version_struct.loader_layer_interface_version = 2;

    // Only vkGetInstanceProcAddr and vkCreateInstance are mandatory to intercept
    version_struct.pfn_get_instance_proc_addr = dummy_vkGetInstanceProcAddr;

    // pfn_get_device_proc_addr and pfn_get_physical_device_proc_addr are optional
    // and can be cleared with related functions & match branches if not needed
    version_struct.pfn_get_device_proc_addr = dummy_vkGetDeviceProcAddr;
    version_struct.pfn_get_physical_device_proc_addr = dummy_vk_layerGetPhysicalDeviceProcAddr;
    vk::Result::SUCCESS
}
const _: PFN_vkNegotiateLoaderLayerInterfaceVersion = vkNegotiateLoaderLayerInterfaceVersion;

#[no_mangle]
unsafe extern "system" fn dummy_vkGetInstanceProcAddr(
    instance: vk::Instance,
    p_name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    let name = CStr::from_ptr(p_name);
    loop {
        let pfn: *const () = match name.to_bytes() {
            b"vkGetInstanceProcAddr" => dummy_vkGetInstanceProcAddr as _,
            b"vkCreateInstance" => dummy_vkCreateInstance as _,
            b"vkDestroyInstance" => dummy_vkDestroyInstance as _,
            b"vkGetDeviceProcAddr" => dummy_vkGetDeviceProcAddr as _,
            b"vkCreateDevice" => dummy_vkCreateDevice as _,
            b"vkDestroyDevice" => dummy_vkDestroyDevice as _,
            b"vk_layerGetPhysicalDeviceProcAddr" => dummy_vk_layerGetPhysicalDeviceProcAddr as _,
            _ => break,
        };
        log!("intercept {}: {:?}", name.to_string_lossy(), pfn);
        return ::core::mem::transmute(pfn);
    }
    let gipa = GIPA.get()?;
    gipa(instance, p_name)
}
const _: vk::PFN_vkGetInstanceProcAddr = dummy_vkGetInstanceProcAddr;

#[no_mangle]
unsafe extern "system" fn dummy_vkGetDeviceProcAddr(
    device: vk::Device,
    p_name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    let name = CStr::from_ptr(p_name);
    loop {
        let pfn: *const () = match name.to_bytes() {
            b"vkGetDeviceProcAddr" => dummy_vkGetDeviceProcAddr as _,
            b"vkCreateDevice" => dummy_vkCreateDevice as _,
            b"vkDestroyDevice" => dummy_vkDestroyDevice as _,
            _ => break,
        };
        log!("intercept {}: {:?}", name.to_string_lossy(), pfn);
        return ::core::mem::transmute(pfn);
    }
    let gdpa = GDPA_MAP.get(&device)?;
    gdpa(device, p_name)
}

const _: vk::PFN_vkGetDeviceProcAddr = dummy_vkGetDeviceProcAddr;

#[no_mangle]
unsafe extern "system" fn dummy_vkCreateInstance(
    p_create_info: *const vk::InstanceCreateInfo,
    p_allocator: *const vk::AllocationCallbacks,
    p_instance: *mut vk::Instance,
) -> vk::Result {
    let create_info = p_create_info.read();
    let chain_info = get_instance_chain_info(&create_info, LayerFunction::LAYER_LINK_INFO);
    let chain_info = if let Some(mut v) = chain_info {
        v.as_mut()
    } else {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    };

    let layer_info = chain_info.u.p_layer_info.read();
    chain_info.u.p_layer_info = layer_info.p_next;

    let gipa = layer_info.pfn_next_get_instance_proc_addr;
    let _ = GPHYPA.set(layer_info.pfn_next_get_physical_device_proc_addr);

    let name = CStr::from_bytes_with_nul_unchecked(b"vkCreateInstance\0");
    let create_instance: vk::PFN_vkCreateInstance =
        mem::transmute(gipa(vk::Instance::null(), name.as_ptr()));

    let res = create_instance(p_create_info, p_allocator, p_instance);
    if res != vk::Result::SUCCESS {
        return res;
    }
    assert!(!p_instance.is_null());

    let instance = *p_instance;

    // IMPORTANT: this should be put before any code executing dispatch_next_vkGetInstanceProcAddr
    //            i.e. ash::Instance::load and khr::Surface::new
    let _ = GIPA.set(gipa);

    let entry = ash::Entry::from_static_fn(vk::StaticFn {
        // IMPORTANT: this make sure the layer provided device specific vkGetDeviceProcAddr is used instead of
        //            the instance specific one get from vkGetInstanceProcAddr, as the later would somehow crashes on execution.
        get_instance_proc_addr: dispatch_next_vkGetInstanceProcAddr,
    });
    let _ = ENTRY.set(entry.clone());

    let ash_instance = ash::Instance::load(entry.static_fn(), instance);
    let khr_surface = khr::Surface::new(&entry, &ash_instance);

    log!("created {:?}", instance);

    let phy_devices = ash_instance.enumerate_physical_devices().unwrap();
    for phy_device in phy_devices {
        PHY_TO_INSTANCE_MAP.insert(phy_device, instance);
    }
    INSTANCE_MAP.insert(
        instance,
        LayerInstance {
            ash_instance,
            khr_surface,
        },
    );

    vk::Result::SUCCESS
}
const _: vk::PFN_vkCreateInstance = dummy_vkCreateInstance;

#[no_mangle]
unsafe extern "system" fn dummy_vkDestroyInstance(
    instance: vk::Instance,
    p_allocator: *const vk::AllocationCallbacks,
) -> () {
    let layer_instance = INSTANCE_MAP.remove(&instance);

    let ash_instance = if let Some(v) = layer_instance {
        v.1.ash_instance
    } else {
        return;
    };

    let phy_devices = ash_instance.enumerate_physical_devices().unwrap();
    for phy_device in phy_devices {
        PHY_TO_INSTANCE_MAP.remove(&phy_device);
    }
    log!("destroying {:?}", instance);
    (ash_instance.fp_v1_0().destroy_instance)(instance, p_allocator);
}
const _: vk::PFN_vkDestroyInstance = dummy_vkDestroyInstance;

#[no_mangle]
unsafe extern "system" fn dummy_vkCreateDevice(
    physical_device: vk::PhysicalDevice,
    p_create_info: *const vk::DeviceCreateInfo,
    p_allocator: *const vk::AllocationCallbacks,
    p_device: *mut vk::Device,
) -> vk::Result {
    log!("physical_device {:?}", physical_device);
    let instance = *PHY_TO_INSTANCE_MAP.get(&physical_device).unwrap();
    let layer_instance = INSTANCE_MAP.get(&instance).unwrap();
    let ash_instance = &layer_instance.ash_instance;
    let instance_fn = ash_instance.fp_v1_0();

    let create_info = p_create_info.read();
    let chain_info = get_device_chain_info(&create_info, LayerFunction::LAYER_LINK_INFO);
    let chain_info = if let Some(mut v) = chain_info {
        v.as_mut()
    } else {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    };

    let layer_info = chain_info.u.p_layer_info.read();
    chain_info.u.p_layer_info = layer_info.p_next;

    let gdpa = layer_info.pfn_next_get_device_proc_addr;

    let res = (instance_fn.create_device)(physical_device, p_create_info, p_allocator, p_device);
    if res != vk::Result::SUCCESS {
        return res;
    }
    assert!(!p_device.is_null());

    let device = *p_device;

    // IMPORTANT: this should be put before any code executing dispatch_next_vkGetDeviceProcAddr,
    //            i.e. `ash::Device::load()` and `khr::Swapchain::new()`
    GDPA_MAP.insert(device, gdpa);

    let ash_device = ash::Device::load(&instance_fn, device);

    log!("created {:?}", device);

    let khr_swapchain = khr::Swapchain::new(ash_instance, &ash_device);

    DEVICE_MAP.insert(
        device,
        LayerDevice {
            instance,
            ash_device,
            khr_swapchain,
        },
    );

    vk::Result::SUCCESS
}
const _: vk::PFN_vkCreateDevice = dummy_vkCreateDevice;

#[no_mangle]
unsafe extern "system" fn dummy_vkDestroyDevice(
    device: vk::Device,
    p_allocator: *const vk::AllocationCallbacks,
) -> () {
    GDPA_MAP.remove(&device);

    let layer_device = DEVICE_MAP.remove(&device);

    let ash_device = if let Some(v) = layer_device {
        v.1.ash_device
    } else {
        return;
    };

    log!("destroying {:?}", device);
    (ash_device.fp_v1_0().destroy_device)(device, p_allocator);
}
const _: vk::PFN_vkDestroyDevice = dummy_vkDestroyDevice;

#[no_mangle]
unsafe extern "system" fn dummy_vk_layerGetPhysicalDeviceProcAddr(
    instance: vk::Instance,
    p_name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    let name = CStr::from_ptr(p_name);
    loop {
        let pfn: *const () = match name.to_bytes() {
            b"vkCreateDevice" => dummy_vkCreateDevice as _,
            _ => break,
        };
        return ::core::mem::transmute(pfn);
    }
    let gphypa = GPHYPA.get()?;
    gphypa(instance, p_name)
}
const _: PFN_vk_layerGetPhysicalDeviceProcAddr = dummy_vk_layerGetPhysicalDeviceProcAddr;

#[no_mangle]
unsafe extern "system" fn dispatch_next_vkGetInstanceProcAddr(
    instance: vk::Instance,
    p_name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    let name = CStr::from_ptr(p_name);
    loop {
        let pfn: *const () = match name.to_bytes() {
            b"vkGetInstanceProcAddr" => dispatch_next_vkGetInstanceProcAddr as _,
            b"vkGetDeviceProcAddr" => dispatch_next_vkGetDeviceProcAddr as _,
            // These would cause some layer (i.e. VK_LAYER_KHRONOS_validation) crashes if called down via vkGetInstanceProcAddr.
            // But as ash::Entry loads all the global functions, we need to force return null to workaround it.
            // If you really need calling down these functions, follow
            // https://vulkan.lunarg.com/doc/view/1.3.236.0/linux/LoaderLayerInterface.html#user-content-pre-instance-functions
            b"vkEnumerateInstanceExtensionProperties" => core::ptr::null(),
            b"vkEnumerateInstanceLayerProperties" => core::ptr::null(),
            b"vkEnumerateInstanceVersion" => core::ptr::null(),
            _ => break,
        };
        return ::core::mem::transmute(pfn);
    }
    let gipa = GIPA.get()?;
    gipa(instance, p_name)
}
const _: vk::PFN_vkGetInstanceProcAddr = dispatch_next_vkGetInstanceProcAddr;

#[no_mangle]
unsafe extern "system" fn dispatch_next_vkGetDeviceProcAddr(
    device: vk::Device,
    p_name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    let name = CStr::from_ptr(p_name);
    loop {
        let pfn: *const () = match name.to_bytes() {
            b"vkGetDeviceProcAddr" => dispatch_next_vkGetDeviceProcAddr as _,
            _ => break,
        };
        return ::core::mem::transmute(pfn);
    }
    let gdpa = GDPA_MAP.get(&device)?;
    gdpa(device, p_name)
}
const _: vk::PFN_vkGetDeviceProcAddr = dispatch_next_vkGetDeviceProcAddr;
