use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::Queue;


pub mod compute_mandelbrot;

pub mod graphics_triangle;


pub fn initialize() -> (Arc<Instance>, Arc<Device>, Arc<Queue>, Arc<Queue>, Arc<Queue>) {
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

    // TODO: pick a dedicated compute and transfer queue if appropriate
    (instance.clone(), device.clone(), queue.clone(), queue.clone(), queue.clone())
}




