use crate::{ HEIGHT, NAME, WIDTH, Pref, };
use ash::{ extensions::{ ext::DebugUtils, khr::{ Surface, Swapchain, DynamicRendering }, }, vk::{ self, SurfaceTransformFlagsKHR }, Device, Entry, Instance, };
use raw_window_handle::{ HasRawDisplayHandle, HasRawWindowHandle };
use std::{ ffi::{ c_void, CStr, CString }, error::Error };
use winit::{ event_loop::EventLoop, monitor::MonitorHandle, window::{ WindowBuilder }, };

pub struct Interface {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_util_loader: DebugUtils,
    pub window: winit::window::Window,
    pub monitor_list: Vec<MonitorHandle>,
    pub monitor: MonitorHandle,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,

    pub phy_device: vk::PhysicalDevice,
    pub physical_device_prop: vk::PhysicalDeviceProperties,
    pub device_memory_prop: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,
    pub surface_capability: vk::SurfaceCapabilitiesKHR,
    pub pre_transform: SurfaceTransformFlagsKHR,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub desired_image_count: u32,
    pub present_mode_list: Vec<vk::PresentModeKHR>,
    pub present_img_list: Vec<vk::Image>,
    pub present_img_view_list: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub draw_command_fence: vk::Fence,
    pub setup_command_fence: vk::Fence,
}

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => { {
            #[allow(unused_unsafe)]
            unsafe {
                let base_zeroed: $base = mem::zeroed();
                std::ptr::addr_of!(base_zeroed.$field) as isize - std::ptr::addr_of!(base_zeroed) as isize
            }
        }
    };
}

unsafe extern "system" fn vulkan_debug_callback(flag: vk::DebugUtilsMessageSeverityFlagsEXT, msg_type: vk::DebugUtilsMessageTypeFlagsEXT, callback_data: * const vk::DebugUtilsMessengerCallbackDataEXT, _: *mut c_void, ) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Flag; 
    let message = CStr::from_ptr((* callback_data).p_message);

    match flag { 
        Flag::VERBOSE => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ),
        Flag::INFO => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ),
        Flag::WARNING => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ),
        _ => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ),
    }

    return vk::FALSE
}


impl Interface {
    pub fn init(event_loop: &EventLoop<()>, pref: &Pref, ) -> Self {
        unsafe {
            log::info!("Creating Window and EventLoop ...");
            let window = WindowBuilder::new()
                .with_title(NAME)
                .with_inner_size(winit::dpi::LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT), ))
                .build(event_loop)
                .unwrap();

            let monitor_list: Vec<MonitorHandle> = event_loop.available_monitors().collect();
            let monitor = monitor_list.first().expect("ERR_NO_MONITOR").clone();
            log::info!("Moniter is [ {} ]", monitor.name().unwrap(),);

            let entry = Entry::load().unwrap();

            log::info!("Creating VulkanInstance ...");
            let name = CString::new(crate::NAME).unwrap();
            let engine_name = CString::new(crate::ENGINE_NAME).unwrap();

            let mut extension_name_list =
                ash_window::enumerate_required_extensions(window.raw_display_handle())
                    .unwrap()
                    .to_vec();
            extension_name_list.push(DebugUtils::name().as_ptr());

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
                extension_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
            }

            let (major, minor) = 
                match entry.try_enumerate_instance_version().unwrap() { 
                    Some(version) => (vk::api_version_major(version), vk::api_version_minor(version), ), 
                    None => (1, 0), 
                };
            
            log::info!("Vulkan {:?}.{:?} supported ...", major, minor, );

            let app_info = vk::ApplicationInfo::builder()
                .application_name(name.as_c_str())
                .application_version(vk::make_api_version(0, 0, 1, 0, ))
                .engine_name(engine_name.as_c_str())
                .engine_version(vk::make_api_version(0, 0, 1, 0, ))
                .api_version(vk::make_api_version(0, major, minor, 0, ));

            let create_flag = 
                if cfg!(any(target_os = "macos", target_os = "ios", )) {
                    vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
                } else {
                    vk::InstanceCreateFlags::default()
                };

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&extension_name_list)
                .flags(create_flag);

            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("ERR_CREATE_INSTANCE");

            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::INFO, )
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE, )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_util_loader = DebugUtils::new(&entry, &instance, );
            let debug_call_back = debug_util_loader
                .create_debug_utils_messenger(&debug_info, None, )
                .unwrap();
            
            let surface = 
                ash_window::create_surface(&entry, &instance, window.raw_display_handle(), window.raw_window_handle(), None, )
                .unwrap();

            log::info!("Creating PhyDevice ...");
            let phy_device_list = instance
                .enumerate_physical_devices()
                .expect("ERR_NO_PHY_DEVICE");

            let surface_loader = Surface::new(&entry, &instance);
            let (phy_device, queue_family_index, ) = phy_device_list
                .iter()
                .find_map(| phy_device | {
                    instance
                        .get_physical_device_queue_family_properties(* phy_device)
                        .iter()
                        .enumerate()
                        .find_map(| (index, info, ) | {
                            let graphic_surface_support =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                        .get_physical_device_surface_support(* phy_device, index as u32, surface, )
                                        .unwrap();
                            if graphic_surface_support { Some((* phy_device, index)) }
                            else { None }
                        })
                })
                .expect("NO_SUITABLE_PHY_DEVICE");
            
            let physical_device_prop = instance.get_physical_device_properties(phy_device);
            let device_memory_prop = instance.get_physical_device_memory_properties(phy_device);
            
            log::info!("Selected PhysicalDevice [ {} ]", &CStr::from_ptr(physical_device_prop.device_name.as_ptr()).to_str().unwrap(), );
            log::info!("Max WorkGroupSize is [ {} x {} x {} ]", physical_device_prop.limits.max_compute_work_group_size[0], physical_device_prop.limits.max_compute_work_group_size[1], physical_device_prop.limits.max_compute_work_group_size[2], );
            log::info!("Max WorkGroupInvocation [ {} ]", physical_device_prop.limits.max_compute_work_group_invocations, );
            log::info!("Max WorkGroupCount is [ {} x {} x {} ]", physical_device_prop.limits.max_compute_work_group_count[0], physical_device_prop.limits.max_compute_work_group_count[1], physical_device_prop.limits.max_compute_work_group_count[2], );

            let queue_family_index = queue_family_index as u32;
            let device_extension_list = [
                Swapchain::name().as_ptr(),
                DynamicRendering::name().as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios", ))]
                KhrPortabilitySubsetFn::name().as_ptr(),
            ];

            let feature = vk::PhysicalDeviceFeatures { shader_clip_distance: 1, ..Default::default() };
            let priority = [1.0];

            log::info!("Get QueueList ...");
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priority);

            let mut dynamic_rendering_feature = vk::PhysicalDeviceDynamicRenderingFeaturesKHR::builder().dynamic_rendering(true);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_list)
                .enabled_features(&feature)
                .push_next(&mut dynamic_rendering_feature);

            let device: Device = instance
                .create_device(phy_device, &device_create_info, None, )
                .unwrap();

            let present_queue = device.get_device_queue(queue_family_index, 0);

            log::info!("Load Surface ...");
            let surface_format = surface_loader
                .get_physical_device_surface_formats(phy_device, surface, )
                .unwrap()[0];

            let surface_capability = surface_loader
                .get_physical_device_surface_capabilities(phy_device, surface, )
                .unwrap();
            
            let mut desired_image_count = surface_capability.min_image_count + 1;
            if surface_capability.max_image_count > 0 && desired_image_count > surface_capability.max_image_count {
                desired_image_count = surface_capability.max_image_count;
            }
            
            let surface_resolution = match surface_capability.current_extent.width {
                std::u32::MAX => vk::Extent2D { width: WIDTH, height: HEIGHT, },
                _ => surface_capability.current_extent,
            };

            let pre_transform = 
                if surface_capability.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
                    vk::SurfaceTransformFlagsKHR::IDENTITY
                } else {
                    surface_capability.current_transform
                };

            let present_mode_list = surface_loader
                .get_physical_device_surface_present_modes(phy_device, surface, )
                .unwrap();
            
            let present_mode = present_mode_list
                .iter()
                .cloned()
                .find(| &mode | mode == pref.pref_present_mode)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            
            log::info!("Creating Swapchain ...");
            let swapchain_loader = Swapchain::new(&instance, &device, );

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None, )
                .unwrap();

            log::info!("Creating CommandPool ...");
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let pool = device.create_command_pool(&pool_create_info, None, ).unwrap();

            log::info!("Creating CommandBuffer ...");
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffer_list = device
                .allocate_command_buffers(&command_buffer_allocate_info, )
                .unwrap();
            
            let setup_command_buffer = command_buffer_list[0];
            let draw_command_buffer = command_buffer_list[1];

            log::info!("Load PresentImgList ...");
            let present_img_list = swapchain_loader
                .get_swapchain_images(swapchain)
                .unwrap();
            let present_img_view_list: Vec<vk::ImageView> = present_img_list
                .iter()
                .map(| &image | {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping { r: vk::ComponentSwizzle::R, g: vk::ComponentSwizzle::G, b: vk::ComponentSwizzle::B, a: vk::ComponentSwizzle::A, })
                        .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, })
                        .image(image);
                    device
                        .create_image_view(&create_view_info, None, )
                        .unwrap()
                })
                .collect();
            
            log::info!("Init Fence ...");
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_command_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");
            let setup_command_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");

            log::info!("Init Semaphore ...");
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            log::info!("Interface finished ...");
            Interface {
                entry,
                instance,
                device,
                queue_family_index,
                phy_device,
                physical_device_prop,
                device_memory_prop,
                window,
                monitor_list,
                monitor,
                surface_loader,
                surface_format,
                present_queue,
                surface_capability,
                pre_transform,
                surface_resolution,
                desired_image_count,
                present_mode_list,
                swapchain_loader,
                swapchain,
                present_img_list,
                present_img_view_list,
                pool,
                draw_command_buffer,
                setup_command_buffer,
                present_complete_semaphore,
                rendering_complete_semaphore,
                draw_command_fence,
                setup_command_fence,
                surface,
                debug_call_back,
                debug_util_loader,
            }
        }
    }

    pub fn find_memorytype_index(&self, memory_req: &vk::MemoryRequirements, flag: vk::MemoryPropertyFlags, ) -> Option<u32> {
        self.device_memory_prop.memory_types[ .. self.device_memory_prop.memory_type_count as _ ]
            .iter()
            .enumerate()
            .find(| (index, memory_type, ) | { (1 << index) & memory_req.memory_type_bits != 0 && memory_type.property_flags & flag == flag })
            .map(| (index, _memory_type, ) | index as _)
    }

    pub fn wait_for_gpu(&self) -> Result<(), Box<dyn Error>> {
        unsafe { Ok(self.device.device_wait_idle().unwrap()) }
    }

}