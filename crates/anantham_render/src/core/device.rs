use ash::{ext, khr, vk};
use bevy::window::RawHandleWrapper;
use std::error::Error;

pub struct VulkanDevice {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub surface: vk::SurfaceKHR,
    pub surface_ext: khr::surface::Instance,
    pub mesh_ext: ext::mesh_shader::Device,
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family_index: u32,
}

impl VulkanDevice {
    pub fn new(handle_wrapper: &RawHandleWrapper) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { ash::Entry::load()? };

        // 1. Setup Instance
        let app_name = c"Anantham Engine";
        let engine_name = c"Anantham Voxel Engine";

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let display_handle = handle_wrapper.get_display_handle();
        let window_handle = handle_wrapper.get_window_handle();

        // Add Debug Utils if needed
        let mut extension_names =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();
        extension_names.push(ash::ext::debug_utils::NAME.as_ptr());

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        // 2. Setup Surface
        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)?
        };
        let surface_ext = khr::surface::Instance::new(&entry, &instance);

        // 3. Select Physical Device
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };
        let (physical_device, queue_family_index) = physical_devices
            .into_iter()
            .find_map(|pdevice| {
                unsafe {
                    let properties = instance.get_physical_device_queue_family_properties(pdevice);
                    for (index, info) in properties.iter().enumerate() {
                        let supports_graphics = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                        let supports_surface = surface_ext
                            .get_physical_device_surface_support(pdevice, index as u32, surface)
                            .unwrap_or(false);

                        if supports_graphics && supports_surface {
                            return Some((pdevice, index as u32));
                        }
                    }
                }
                None
            })
            .expect("Failed to find a suitable Vulkan physical device.");

        // 4. Create Logical Device
        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_extension_names_raw = [
            khr::swapchain::NAME.as_ptr(),
            ext::mesh_shader::NAME.as_ptr(),
        ];

        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let mut mesh_shader_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
            .mesh_shader(true)
            .task_shader(true);

        let mut features2 = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vulkan_13_features)
            .push_next(&mut mesh_shader_features);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .push_next(&mut features2);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        let mesh_ext = ext::mesh_shader::Device::new(&instance, &device);

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            surface,
            surface_ext,
            mesh_ext,
            graphics_queue,
            graphics_queue_family_index: queue_family_index,
        })
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            // Clean up the massive driver allocations
            self.device.destroy_device(None);
            self.surface_ext.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
