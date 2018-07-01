use std::sync::Arc;

use vulkano_win;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Queue;
use vulkano::instance;
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::LayerProperties;
use vulkano::instance::PhysicalDevice;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;
use vulkano::format::Format;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;

pub mod compute_mandelbrot;

pub mod graphics_triangle;


pub fn make_img_and_buf(device: Arc<Device>, queue: Arc<Queue>, size: u32) ->
        (Arc<StorageImage<Format>>, Arc<CpuAccessibleBuffer<[u8]>>) {

    let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: size, height: size },
                                  Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();

    let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                             (0..size * size * 4).map(|_| 0u8))
        .expect("failed to create buffer");

    (image, buf)
}

pub fn initialize() -> (Arc<Instance>, Arc<Device>, Arc<Queue>, Arc<Queue>, Arc<Queue>) {
    let extensions = InstanceExtensions {
        ext_debug_report: true,
        ..vulkano_win::required_extensions()
    };

    let all_layers: Vec<LayerProperties> = instance::layers_list().unwrap().collect();

    for layer in &all_layers {
        println!("{:?} - {:?}", layer.name(), layer.description());
    }

    let chosen_layers: Vec<String> = all_layers.iter()
            .filter(|l| l.name().contains("validation"))
            .map(|l| String::from(l.name()))
            .collect();

    for layer in &chosen_layers {
        println!("Validation: {:?}", layer);
    }

    let used_layers: Vec<&str> = chosen_layers.iter().map(|ln| ln.as_str()).collect();

    let instance = Instance::new(
        None,
        &extensions,
        used_layers.iter())
        .expect("failed to create instance");


    let all = MessageTypes {
        error: true,
        warning: true,
        performance_warning: true,
        information: true,
        debug: false,
    };

    let _debug_callback = DebugCallback::new(&instance, all, |msg| {
        let ty = if msg.ty.error {
            "error"
        } else if msg.ty.warning {
            "warning"
        } else if msg.ty.performance_warning {
            "performance_warning"
        } else if msg.ty.information {
            "information"
        } else if msg.ty.debug {
            "debug"
        } else {
            panic!("no-impl");
        };
        println!("{} {}: {}", msg.layer_prefix, ty, msg.description);
    }).ok();


    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s). Graphics: {:?}, Compute: {:?}, Transfer: {:?}",
                 family.queues_count(), family.supports_graphics(), family.supports_compute(), family.supports_transfers());
    }

    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics() && q.supports_transfers())
        .expect("couldn't find a graphical queue family");


    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
                    [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };

    let queue = queues.next().unwrap();

    // TODO: pick a dedicated compute and transfer queue if appropriate
    (instance.clone(), device.clone(), queue.clone(), queue.clone(), queue.clone())
}




