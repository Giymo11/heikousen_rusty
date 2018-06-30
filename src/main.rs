#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

extern crate image;


mod engine;



fn main() {
    println!("Hello, world!");

    let (instance, device, allround_queue, compute_queue, transfer_queue) = engine::initialize();

    engine::compute_mandelbrot::make_mandelbrot(device.clone(), allround_queue.clone(), 1024, "image2.png");

    engine::graphics_triangle::make_triangle(device.clone(), allround_queue.clone(), 1024, "triangle.png");

    // TODO: extract the <image creation and the writing of the buffer to disk> into separate functions
}
