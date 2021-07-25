mod rmq;
mod gst;

use rmq::rmq_runner;
use futures::{join};

#[tokio::main]
pub async fn tokio_main(){
    let _ = join!(rmq_runner());
}