use gstreamer as gst;
use gstreamer_editing_services as ges;

pub fn clip_video() {
    println!("Hello, world!");
    ges::init();

    let timeline = ges::Timeline::new_audio_video();
    let layer = timeline.append_layer();

    let pipeline = ges::Pipeline::new();
    pipeline.set_timeline(&timeline)?;

    let clip = ges::UriClip::new("file:///mnt/f/Personal-Docs/Repos/auto-highlighter-processing-service/input/test-video.mp4").expect("Failed to create clip");
    layer.add_clip(&clip)?;

    let asset = clip.asset().unwrap();
    let duration = asset.downcast::<ges::UriClipAsset>().unwrap().duration();
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

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
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
