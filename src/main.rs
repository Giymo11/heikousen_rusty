#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

extern crate image;


mod engine;


use std::sync::Arc;


fn main() {
    println!("Hello, world!");

    let (instance, device, queue) = engine::initialize();

    engine::make_mandelbrot(device.clone(), queue.clone(), 1024, "image2.png");



}
