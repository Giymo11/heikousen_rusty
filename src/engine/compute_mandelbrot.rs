mod cs {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[src = "
    #version 450

    layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

    layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

    void main() {
        vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
        vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

        vec2 z = vec2(0.0, 0.0);
        float i;
        for (i = 0.0; i < 1.0; i += 0.005) {
            z = vec2(
                z.x * z.x - z.y * z.y + c.x,
                z.y * z.x + z.x * z.y + c.y
            );

            if (length(z) > 4.0) {
                break;
            }
        }

        vec4 to_write = vec4(vec3(i), 1.0);
        imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
    }"
    ]
    struct Dummy;
}

use std::sync::Arc;

use vulkano::device::Device;
use vulkano::device::Queue;

pub fn make_mandelbrot(device: Arc<Device>, queue: Arc<Queue>, size: u32, path: &str) {
    let shader = cs::Shader::load(device.clone())
        .expect("failed to create shader module");

    use vulkano::pipeline::ComputePipeline;

    let compute_pipeline = Arc::new(
        ComputePipeline::new(
            device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline"));


    use vulkano::format::Format;
    use vulkano::image::Dimensions;
    use vulkano::image::StorageImage;

    let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: size, height: size },
                                  Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();


    use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

    let set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
        .add_image(image.clone()).unwrap()
        .build().unwrap()
    );


    use vulkano::buffer::CpuAccessibleBuffer;
    use vulkano::buffer::BufferUsage;

    let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                             (0 .. size * size * 4).map(|_| 0u8))
        .expect("failed to create buffer");


    use vulkano::command_buffer::AutoCommandBufferBuilder;

    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
        .dispatch([size / 8, size / 8, 1], compute_pipeline.clone(), set.clone(), ()).unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone()).unwrap()
        .build().unwrap();


    use vulkano::command_buffer::CommandBuffer;
    use vulkano::sync::GpuFuture;

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap()
        .wait(None).unwrap();


    use image::{ImageBuffer, Rgba};

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(size, size, &buffer_content[..]).unwrap();
    image.save(path).unwrap();
}