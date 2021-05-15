use gstreamer as gst;
use gstreamer::{ElementExt, ElementExtManual, GstObjectExt};
use gstreamer_editing_services as ges;
use gstreamer_editing_services::{GESPipelineExt, LayerExt, TimelineElementExt, TimelineExt};

pub fn clip_video() {
    println!("Hello, world!");
    match ges::init() {
        Err(e) => println!("{:?}", e),
        _ => (),
    }

    let timeline = ges::Timeline::new_audio_video();
    let layer = timeline.append_layer();

    let pipeline = ges::Pipeline::new();
    match pipeline.set_timeline(&timeline) {
        Err(e) => println!("{:?}", e),
        _ => (),
    }

    let clip = ges::UriClip::new("file:///mnt/f/Personal-Docs/Repos/auto-highlighter-processing-service/input/test-video.mp4").expect("Failed to create clip");
    match layer.add_clip(&clip) {
        Err(e) => println!("{:?}", e),
        _ => (),
    }

    let duration = clip.get_duration();
    println!(
        "Clip duration: {} - playing file from {} for {}",
        duration,
        duration / 2,
        duration / 4
    );

    clip.set_inpoint(duration / 2);
    clip.set_duration(duration / 4);

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    let bus = pipeline.get_bus().unwrap();

    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
                break;
            }
            _ => (),
        }
    }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}
