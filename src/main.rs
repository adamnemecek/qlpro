use core_foundation::*;
use core_graphics::event::CGEventTap;

fn main() {
    let a= CGEventTap::new(|proxy,type_,event|{

    });
    println!("Hello, world!");
}
