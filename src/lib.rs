use gstreamer;
use gstreamer_editing_services;

pub fn clip_video(){
    gstreamer::init().unwrap();
    gstreamer_editing_services::init().unwrap();
    println!("Hello, world!");
}