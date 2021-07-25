use gstreamer as gst;
use gstreamer::{ElementExt, ElementExtManual, GstObjectExt};
use gstreamer_editing_services as ges;
use gstreamer_editing_services::{GESPipelineExt, LayerExt, TimelineExt};
use gstreamer_pbutils as gst_pbutils;
use gstreamer_pbutils::EncodingProfileBuilder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug)]
pub struct HighlightConfig {
    hid: Uuid,
    time_stamps: Vec<i32>,
    clip_duraiton: i32,
}

pub fn ges_init(){
    match ges::init() {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }
}

pub fn make_highlights(config: HighlightConfig) {

    let (_timeline, pipeline, layer) = init_processing_structs();

    let contianer_profile = get_container_profile();

    let asset = ges::UriClipAsset::request_sync(
        "file:///home/ryan/repos/auto-highlighter-processing-service/input/test-video-sot.mp4",
    )
    .expect("Failed to create asset");

    match layer.add_asset(
        &asset,
        0 * gst::SECOND,
        0 * gst::SECOND,
        10 * gst::SECOND,
        ges::TrackType::UNKNOWN,
    ) {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }

    match pipeline.set_render_settings(
        "file:///home/ryan/repos/auto-highlighter-processing-service/output/test-video-sot.mp4",
        &contianer_profile,
    ) {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }

    render_video(pipeline);
}

pub fn init_processing_structs() -> (ges::Timeline, ges::Pipeline, ges::Layer) {
    let timeline = ges::Timeline::new_audio_video();

    let pipeline = ges::Pipeline::new();
    match pipeline.set_timeline(&timeline) {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }

    let layer = timeline.append_layer();

    return (timeline, pipeline, layer);
}

fn render_video(pipeline: ges::Pipeline) {
    match pipeline.set_mode(ges::PipelineFlags::SMART_RENDER) {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }

    pipeline
        .set_state(gst::State::Ready)
        .expect("Unable to set the pipeline to the `Ready` state");

    pipeline
        .set_state(gst::State::Paused)
        .expect("Unable to set the pipeline to the `Ready` state");

    match pipeline.set_state(gst::State::Playing) {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }

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
        .set_state(gst::State::Paused)
        .expect("Unable to set the pipeline to the `Ready` state");

    pipeline
        .set_state(gst::State::Ready)
        .expect("Unable to set the pipeline to the `Ready` state");

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}

fn get_container_profile() -> gst_pbutils::EncodingContainerProfile {

    let video_profile = gst_pbutils::EncodingVideoProfileBuilder::new()
        .name("h.264")
        .description("h.264-profile")
        .format(&gst::Caps::new_simple("video/x-h264", &[("framerate", &gst::Fraction::new(30, 1))]))
        .preset(&"Profile YouTube")
        .build()
        .unwrap();

    let audio_profile = gst_pbutils::EncodingAudioProfileBuilder::new()
        .name("mp3")
        .description("mp3-profile")
        .format(&gst::caps::Caps::new_simple(
            "audio/mpeg",
            &[("mpegversion", &1i32), ("layer", &3i32)],
        ))
        .build()
        .unwrap();

    let contianer_profile = gst_pbutils::EncodingContainerProfileBuilder::new()
        .name("default-mp4-profile")
        .description("mp4-with-h.264-mp3")
        .format(&gst::caps::Caps::new_simple(
            "video/quicktime",
            &[("variant", &"iso")],
        ))
        .enabled(true)
        .add_profile(&video_profile)
        .add_profile(&audio_profile)
        .build()
        .unwrap();

    return contianer_profile;
}
