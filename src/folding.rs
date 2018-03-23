#![cfg_attr(
    not(any(feature = "vulkan", feature = "dx12", feature = "metal")),
    allow(dead_code, unused_extern_crates, unused_imports)
)]
#![cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]

use hal;
use hal::{
    Backend, Compute, Device, DescriptorPool, Instance, PhysicalDevice, QueueFamily,
};
use hal::{queue, pso, memory, buffer, pool, command};

use back;

use filters::Palette;
use imaging::Image;
use std::collections::HashMap;

/**
 * A struct which uniquely describes how to build a computation stage.
 * This will contain the input image names and the output image names as well as the
 * intended transformation of those images.
 */
pub struct StageDesc
{
    pub input_ids  : Vec<String>,
    pub output_id  : String,
    pub palette    : Palette,
}

/**
 * Struct which holds the compute shading state
 */
pub struct FoldingMachine<B : Backend, C>
{
    stages  : Vec<StageDesc>,
    memory_properties : hal::MemoryProperties,
    device  : B::Device,
    queue_group : hal::QueueGroup<B, C>,

    images  : HashMap<String, Image>
}

fn create_buffer<B: Backend>(device: &mut B::Device, memory_types: &[hal::MemoryType],
                             properties: memory::Properties, usage: buffer::Usage,
                             stride: u64, len: u64) -> (B::Memory, B::Buffer)
{
    let buffer = device.create_buffer(stride * len, usage).unwrap();
    let requirements = device.get_buffer_requirements(&buffer);

    let ty = memory_types
        .into_iter()
        .enumerate()
        .position(|(id, memory_type)| {
            requirements.type_mask & (1 << id) != 0 &&
            memory_type.properties.contains(properties)
        })
        .unwrap()
        .into();

    let memory = device.allocate_memory(ty, requirements.size).unwrap();
    let buffer = device.bind_buffer_memory(&memory, 0, buffer).unwrap();

    (memory, buffer)
}

impl<B : Backend, C> FoldingMachine<B, C>
{
    pub fn new(image_vec : Vec<(String, String)>, stages : Vec<StageDesc>)
                    -> FoldingMachine<<back::Instance as hal::Instance>::Backend, hal::Compute>
    {
        let instance = back::Instance::create("gfx-rs compute", 1);

        let adapter = instance.enumerate_adapters().into_iter()
                                .find(|a| a.queue_families.iter()
                                    .any(|family| family.supports_compute()))
                                        .expect("Failed to find GPU with compute support.");


        let memory_properties = adapter.physical_device.memory_properties();
        let (device, queue_group) = adapter.open_with::<_, Compute>(1, |_family| true)
                                                .unwrap();

        let mut images = HashMap::new();
        for (name, location) in image_vec
        {
            images.insert(name.clone(), Image::new(name.clone(), location.clone()));
        }

        FoldingMachine { stages, memory_properties, device, queue_group, images }
    }

    pub fn compute_stage(&mut self, stage : usize) -> Result<(), &'static str>
        where C: hal::Supports<hal::Transfer> + hal::Supports<hal::Compute>,
              (hal::Transfer, C) : hal::queue::capability::Upper,
              C: hal::Supports<<(hal::Transfer, C) as hal::queue::capability::Upper>::Result>
    {
        let stage = &self.stages[stage];

        // ========================================================================================
        // == Load images into memory and create the color loading string for the shader         ==
        // ========================================================================================
        let mut image_data : Vec<Vec<u32>> = Vec::new();
        let mut dim = None;
        let mut img_len = 0;
        let out_name = stage.output_id.clone();

        let mut entry_string = format!(
"void main()
{{
  uint index = gl_GlobalInvocationID.x;
  float total = 0;
  vec3  total_vec = vec3(0);
  int   num_zeros = 0;
  vec3  num_zeros_vec = vec3(0);
  vec3  {}_vec = vec3(0);
",
                                        out_name);

        // Gather all images into their data arrays and check that all images have correct size.
        for (i, name) in stage.input_ids.iter().enumerate()
        {
            // Load the image into memory
            let image = self.images.get_mut(name).unwrap();
            match image.load_u32_vec()
            {
                Err(_) => {return Err("Could not load image");},
                Ok(data) =>
                {
                    image_data.push(data);
                }
            }

            // Check the dimensions
            let (x_, y_) = image.data.clone().unwrap().dimensions();
            if dim.is_none()
            {
                dim = Some((x_, y_));
                img_len = x_ * y_;
            }
            else if let Some((x, y)) = dim
            {
                if x != x_ || y != y_
                {
                    return Err("Not all images are same length");
                }
            }

            // Create the shader string which loads the hsv vector for this image at a given pixel
            entry_string.push_str(format!(
"  uint {name}_value = in_colors0[index + {offset}];
  vec4 {name}_color =  vec4(uvec4({name}_value & 255, ({name}_value >> 8) & 255,
                                   ({name}_value >> 16) & 255, ({name}_value >> 24) & 255))/ 255.0;
  vec3 {name}_vec = hsv2half_spherical(rgb2hsv({name}_color.xyz));",
                                     name=name, offset=(i as u32)*img_len).as_str());
        }

        // ========================================================================================
        // == Build the shader strings and the output line                                       ==
        // ========================================================================================

        let start_name = stage.input_ids[0].clone();
        let middle_string = stage.palette.shader(format!("{}_vec", start_name),
                                                 format!("{}_vec", out_name),
                                                 "total".to_owned(),      "num_zeros".to_owned());
        let end_string = format!(
"  // Convert the out_color back into rgb. Maintain alpha.
  vec4 color_out = vec4(hsv2rgb(half_spherical2hsv({}_vec)),
                      {in_name}_color.w);
  uvec4 out_components = uvec4(255 * color_out);
  in_colors0[index] = out_components.x         | (out_components.y << 8) |
                      (out_components.z << 16) | (out_components.w << 24);
}}
",
            out_name=out_name, in_name=start_name
        );

        use std::str;
        let shader_string = String::from(str::from_utf8(include_bytes!("../shaders/lib.comp")).unwrap()) +
                            &entry_string + &middle_string + &end_string;

        // Compile the shader now that we have fully created it.

        {
            use std::fs::File;
            use std::io::prelude::*;
            let mut out_shader = File::create("shaders/tmp.comp").expect("Could not save shader");
            out_shader.write_all(shader_string.as_bytes());
        }

        use glsl_to_spirv::ShaderType;
        use glsl_to_spirv;
        let mut compiled_spriv = glsl_to_spirv::compile(&shader_string, ShaderType::Compute)
                                    .expect("Could not compile shader");
        let mut compiled_contents = Vec::new();
        // use std::fs::File;
        use std::io::Read;
        compiled_spriv.read_to_end(&mut compiled_contents).unwrap();
        let shader = self.device.create_shader_module(compiled_contents.as_slice()).unwrap();

        // ========================================================================================
        // == Create the compute pipeline                                                        ==
        // ========================================================================================

        let (pipeline_layout, pipeline, set_layout, mut desc_pool) = {
            // We have one descriptor for all of the images
            let set_layout = self.device.create_descriptor_set_layout(&[
                    pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: pso::DescriptorType::StorageBuffer,
                        count: 1,
                        stage_flags: pso::ShaderStageFlags::COMPUTE,
                    },
                ],
            );

            // We build the pipeline
            let pipeline_layout = self.device.create_pipeline_layout(Some(&set_layout), &[]);
            let entry_point = pso::EntryPoint { entry: "main", module: &shader, specialization: &[] };
            let pipeline = self.device
                .create_compute_pipeline(&pso::ComputePipelineDesc::new(entry_point, &pipeline_layout))
                .expect("Error creating compute pipeline!");

            // Get the descriptor pool
            let desc_pool = self.device.create_descriptor_pool(
                1,
                &[
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::StorageBuffer,
                        count: 1,
                    },
                ],
            );
            (pipeline_layout, pipeline, set_layout, desc_pool)
        };

        // ========================================================================================
        // == Create the buffers                                                                 ==
        // ========================================================================================

        use std;
        let img_len    : u64 = img_len as u64;
        let stride     : u64 = std::mem::size_of::<u32>() as u64;
        let num_images : u64 = image_data.len() as u64;

        // Create a buffer which can hold the data of all the images.
        let (staging_memory, staging_buffer) = create_buffer::<B>(
            &mut self.device,
            &self.memory_properties.memory_types,
            memory::Properties::CPU_VISIBLE | memory::Properties::COHERENT,
            buffer::Usage::TRANSFER_SRC | buffer::Usage::TRANSFER_DST,
            stride,
            img_len * num_images);

        // Write each image to the buffer in order.
        for (i, data) in image_data.iter().enumerate()
        {
            let start_index = (i as u64) * stride * img_len;
            let end_index   = (i as u64 + 1) * stride * img_len;
            let mut writer = self.device.acquire_mapping_writer::<u32>(&staging_memory, start_index..end_index).unwrap();
            writer.copy_from_slice(data.as_slice());
            self.device.release_mapping_writer(writer);
        }

        // Create the memory which the gpu will compute on.
        let (compute_memory, compute_buffer) = create_buffer::<B>(
            &mut self.device,
            &self.memory_properties.memory_types,
            memory::Properties::DEVICE_LOCAL,
            buffer::Usage::TRANSFER_SRC | buffer::Usage::TRANSFER_DST | buffer::Usage::STORAGE,
            stride,
            img_len * num_images,
        );

        // Create the descriptors
        let desc_set = desc_pool.allocate_set(&set_layout);
        self.device.write_descriptor_sets(Some(
            pso::DescriptorSetWrite {
                set: &desc_set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(
                    pso::Descriptor::Buffer(&compute_buffer, None .. None)
                ),
        }));

        // ========================================================================================
        // == Setup and run the compute shader                                                   ==
        // ========================================================================================

        // Build the command pool and create the memory fence
        let mut command_pool = self.device.create_command_pool_typed(&self.queue_group, pool::CommandPoolCreateFlags::empty(), 16);
        let fence = self.device.create_fence(false);

        // Build the gpu command submission
        let submission = queue::Submission::new().submit(Some(
        {
            // Get the command buffer and copy from staging memory to the compute memory
            let mut command_buffer = command_pool.acquire_command_buffer(false);
            command_buffer.copy_buffer(&staging_buffer, &compute_buffer, &[command::BufferCopy { src: 0, dst: 0, size: num_images * stride * img_len}]);
            // Wait for the data transfer to complete
            command_buffer.pipeline_barrier(
                pso::PipelineStage::TRANSFER .. pso::PipelineStage::COMPUTE_SHADER,
                memory::Dependencies::empty(),
                Some(memory::Barrier::Buffer {
                    states: buffer::Access::TRANSFER_WRITE .. buffer::Access::SHADER_READ | buffer::Access::SHADER_WRITE,
                    target: &compute_buffer
                }),
            );

            // Bind the shader and its descriptors
            command_buffer.bind_compute_pipeline(&pipeline);
            command_buffer.bind_compute_descriptor_sets(&pipeline_layout, 0, &[desc_set]);

            // We then run the shader on each pixel of the image
            command_buffer.dispatch([img_len as u32, 1, 1]);
            // Wait for the shader to complete
            command_buffer.pipeline_barrier(
                pso::PipelineStage::COMPUTE_SHADER .. pso::PipelineStage::TRANSFER,
                memory::Dependencies::empty(),
                Some(memory::Barrier::Buffer {
                    states: buffer::Access::SHADER_READ | buffer::Access::SHADER_WRITE .. buffer::Access::TRANSFER_READ,
                    target: &compute_buffer
                }),
            );

            // Copy only the top image from the compute buffer to the staging memory
            command_buffer.copy_buffer(&compute_buffer, &staging_buffer, &[command::BufferCopy { src: 0, dst: 0, size: stride * img_len as u64}]);
            // Wait for everything to complete.
            command_buffer.finish()
        }));

        // Sumbit the operation and wait for it to complete.
        self.queue_group.queues[0].submit(submission, Some(&fence));
        self.device.wait_for_fence(&fence, !0);

        // ========================================================================================
        // == Load back in the output image and save it                                          ==
        // ========================================================================================

        if let Some((width, height)) = dim
        {
            let reader = self.device.acquire_mapping_reader::<u32>(&staging_memory, 0..stride * img_len as u64).unwrap();
            let new_image_data = reader.into_iter().map(|n| *n).collect::<Vec<u32>>();
            let mut out_image = self.images.get_mut(&stage.output_id).unwrap();
            out_image.save_u32_vec(new_image_data, width, height).expect("Could not save");
            self.device.release_mapping_reader(reader);
        }

        // ========================================================================================
        // == Clear the memory for this compute shader                                           ==
        // ========================================================================================

        self.device.destroy_command_pool(command_pool.downgrade());
        self.device.destroy_descriptor_pool(desc_pool);
        self.device.destroy_descriptor_set_layout(set_layout);
        self.device.destroy_shader_module(shader);
        self.device.destroy_buffer(compute_buffer);
        self.device.destroy_buffer(staging_buffer);
        self.device.destroy_fence(fence);
        self.device.destroy_pipeline_layout(pipeline_layout);
        self.device.free_memory(compute_memory);
        self.device.free_memory(staging_memory);
        self.device.destroy_compute_pipeline(pipeline);

        Ok(())

    }
}