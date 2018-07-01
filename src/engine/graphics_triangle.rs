use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;


mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    ]
    struct Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"]
    struct Dummy;
}


#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl_vertex!(Vertex, position);


use vulkano::image::StorageImage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::format::FormatDesc;
use vulkano::memory::Content;

use vulkano::image::Dimensions;
use vulkano::format::Format;

use vulkano::buffer::BufferUsage;

/*
fn make_image_and_buf<T, F>(device: Arc<Device>, queue: Arc<Queue>, size: u32) ->
    (Arc<StorageImage<F>>, Arc<CpuAccessibleBuffer<[T]>>)
    where F: FormatDesc, T: Content + 'static {

    use vulkano::image::Dimensions;
    use vulkano::format::Format;

    let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: size, height: size },
                                  Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();


    use vulkano::buffer::BufferUsage;

    let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                             (0..size * size * 4).map(|_| 0u8))
        .expect("failed to create buffer");


    (image, buf)
}
*/

pub fn make_triangle(device: Arc<Device>, queue: Arc<Queue>, size: u32, path: &str) {
    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    // let (image, buf) = make_image_and_buf(device.clone(), queue.clone(), size);

    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d { width: size, height: size },
        Format::R8G8B8A8Unorm,
        Some(queue.family()))
        .unwrap();


    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        (0 .. size * size * 4).map(|_| 0u8))
        .expect("failed to create buffer");



    let render_pass = Arc::new(single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());


    use vulkano::framebuffer::Framebuffer;

    let framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
        .add(image.clone()).unwrap()
        .build().unwrap());


    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [0.0, 0.5] };
    let vertex3 = Vertex { position: [0.5, -0.25] };


    let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                                       vec![vertex1, vertex2, vertex3].into_iter()).unwrap();


    use vulkano::pipeline::GraphicsPipeline;
    use vulkano::framebuffer::Subpass;

    let pipeline = Arc::new(GraphicsPipeline::start()
        // Defines what kind of vertex input is expected.
        .vertex_input_single_buffer::<Vertex>()
        // The vertex shader.
        .vertex_shader(vs.main_entry_point(), ())
        // Defines the viewport (explanations below).
        .viewports_dynamic_scissors_irrelevant(1)
        // The fragment shader.
        .fragment_shader(fs.main_entry_point(), ())
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(device.clone())
        .unwrap());


    use vulkano::command_buffer::DynamicState;
    use vulkano::pipeline::viewport::Viewport;

    let dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [size as f32, size as f32],
            depth_range: 0.0..1.0,
        }]),
        ..DynamicState::none()
    };


    use vulkano::command_buffer::AutoCommandBufferBuilder;

    let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
        .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
        .unwrap()

        .draw(pipeline.clone(), dynamic_state, vertex_buffer.clone(), (), ())
        .unwrap()

        .end_render_pass()
        .unwrap()

        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap()

        .build()
        .unwrap();

    use vulkano::command_buffer::CommandBuffer;
    use vulkano::sync::GpuFuture;

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap()
        .wait(None).unwrap();


    use image::ImageBuffer;
    use image::Rgba;

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(size, size, &buffer_content[..]).unwrap();
    image.save("triangle.png").unwrap();
}