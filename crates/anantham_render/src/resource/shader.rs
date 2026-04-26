use ash::vk;
use std::error::Error;

pub struct ShaderModule {
    pub module: vk::ShaderModule,
}

impl ShaderModule {
    /// Loads a pre-compiled SPIR-V shader directly from embedded bytes.
    pub fn new(device: &ash::Device, spv_bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut cursor = std::io::Cursor::new(spv_bytes);
        let spv_words = ash::util::read_spv(&mut cursor)?;

        let create_info = vk::ShaderModuleCreateInfo::default().code(&spv_words);
        let module = unsafe { device.create_shader_module(&create_info, None)? };

        Ok(Self { module })
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_shader_module(self.module, None);
        }
    }
}
