use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::Queue;

pub fn initialize() -> (Arc<Instance>, Arc<Device>, Arc<Queue>) {
    use vulkano::instance::InstanceExtensions;

    let instance = Instance::new(
        None,
        &InstanceExtensions::none(),
        None)
        .expect("failed to create instance");

    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s). Graphics: {:?}, Compute: {:?}, Transfer: {:?}",
                 family.queues_count(), family.supports_graphics(), family.supports_compute(), family.supports_transfers());
    }

    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    use vulkano::device::DeviceExtensions;
    use vulkano::instance::Features;

    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
                    [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };

    let queue = queues.next().unwrap();

    (instance.clone(), device.clone(), queue.clone())
}


mod compute_shader_mandelbrot;

pub fn make_mandelbrot(device: Arc<Device>, queue: Arc<Queue>, size: u32, path: &str) {
    let shader = compute_shader_mandelbrot::Shader::load(device.clone())
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
