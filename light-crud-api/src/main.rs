mod database;

mod lights;

mod config;
use config::read_or_create_config;
use database::frame::Frame;
use lights::converter;

#[tokio::main]
async fn main() {
    let path = "config.toml";
    let config = read_or_create_config(path).unwrap();
    println!("{config:?}");

    if config.debug.enable_webserver {
        let app = database::initialize::setup(&config).await;

        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", config.web.interface, config.web.port))
                .await
                .unwrap();
        axum::serve(listener, app).await.unwrap();
    }
    if config.debug.enable_lights {
        let mut controller = lights::controller::setup(&config);
        let mut test_frame = Frame::new();
        test_frame.data = String::from("[255, 65280, 16711680]");
        lights::controller::write_frame(&test_frame, &mut controller);
    }

}
