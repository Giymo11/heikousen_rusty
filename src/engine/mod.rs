use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::instance::LayerProperties;

pub mod compute_mandelbrot;

pub mod graphics_triangle;


pub fn initialize() -> (Arc<Instance>, Arc<Device>, Arc<Queue>, Arc<Queue>, Arc<Queue>) {

    use vulkano::instance::InstanceExtensions;
    use vulkano_win;

    let extensions = InstanceExtensions {
        ext_debug_report: true,
        ..vulkano_win::required_extensions()
    };


    use vulkano::instance;

    println!("List of Vulkan debugging layers available to use:");
    let layers: Vec<_> = instance::layers_list().unwrap().collect();

    for l in &layers {
        println!("\t{}", l.name());
    }


    let layer = layers.iter().find(|ref x| x.name() == "VK_LAYER_LUNARG_standard_validation");

    match layer {
        Some(layer) => println!("Found debug layer:\n\t{}\n", layer.name()),
        None => println!("Found no debug layer"),
    }

    let used_layers = layer.map(|ref x| x.name().to_owned().as_str());

    let instance = Instance::new(
        None,
        &extensions,
        used_layers)
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

    // TODO: pick a dedicated compute and transfer queue if appropriate
    (instance.clone(), device.clone(), queue.clone(), queue.clone(), queue.clone())
}




