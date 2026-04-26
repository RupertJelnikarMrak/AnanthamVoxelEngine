use crate::resource::allocator::GpuAllocator;
use crate::resource::buffer::Buffer;
use ash::vk;

pub struct GpuVoxelScene {
    pub meshlet_buffer: Buffer,
    pub quad_buffer: Buffer,
    pub meshlet_count: u32,
}

impl GpuVoxelScene {
    pub fn new(
        device: &ash::Device,
        allocator: &mut GpuAllocator,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let quad_buffer = Buffer::new(
            device,
            allocator,
            64 * 1024 * 1024,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
            "Global Quad SSBO",
        )?;

        let meshlet_buffer = Buffer::new(
            device,
            allocator,
            16 * 1024 * 1024,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
            "Global Meshlet SSBO",
        )?;

        Ok(Self {
            meshlet_buffer,
            quad_buffer,
            meshlet_count: 0,
        })
    }

    pub fn destroy(self, device: &ash::Device, allocator: &mut GpuAllocator) {
        self.quad_buffer.destroy(device, allocator);
        self.meshlet_buffer.destroy(device, allocator);
    }
}
