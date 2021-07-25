mod rmq;
mod gst_highlighter;

use gst_highlighter::ges_init;
use rmq::rmq_runner;
use futures::{join};

#[tokio::main]
pub async fn tokio_main(){
    ges_init();
    let _ = join!(rmq_runner());
}