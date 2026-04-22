use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};
use std::error::Error;

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: Allocation,
    pub size: u64,
}

impl Buffer {
    pub fn new(
        device: &ash::Device,
        allocator: &mut Allocator,
        size: u64,
        usage: vk::BufferUsageFlags,
        memory_location: gpu_allocator::MemoryLocation,
        name: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location: memory_location,
            linear: true,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
        }

        Ok(Self {
            buffer,
            allocation,
            size,
        })
    }

    /// Writes data directly if the buffer is CPU-visible (MemoryLocation::CpuToGpu).
    pub fn write_mapped<T: bytemuck::Pod>(&mut self, data: &[T]) {
        if let Some(mapped_ptr) = self.allocation.mapped_ptr() {
            let bytes = bytemuck::cast_slice(data);
            assert!(bytes.len() as u64 <= self.size, "Data exceeds buffer size!");
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    mapped_ptr.as_ptr() as *mut u8,
                    bytes.len(),
                );
            }
        } else {
            panic!("Attempted to write to an unmapped buffer!");
        }
    }

    pub fn destroy(self, device: &ash::Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
        }
        allocator.free(self.allocation).unwrap();
    }
}
