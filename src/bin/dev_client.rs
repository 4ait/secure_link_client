use std::env;
use tokio::runtime::Runtime;
use secure_link_client::SecureLink;

fn main() {

    env_logger::init();
    
    dotenv::dotenv().ok();

    let auth_token = env::var("AUTH_TOKEN")
        .expect("AUTH_TOKEN environment variable is required");

    let secure_link_server_host = env::var("SECURE_LINK_SERVER_HOST")
        .expect("SECURE_LINK_SERVER_HOST environment variable is required");

    let secure_link_server_port: u16 = env::var("SECURE_LINK_SERVER_PORT")
        .expect("SECURE_LINK_SERVER_PORT environment variable is required")
        .parse()
        .expect("SECURE_LINK_SERVER_PORT must be a valid port number");

    Runtime::new().unwrap().block_on(async {

        let secure_link_connection_result =
            SecureLink::connect_to_global_channel(
                &secure_link_server_host,
                secure_link_server_port,
                &auth_token
            ).await.unwrap();
        
        let res = secure_link_connection_result.run_message_loop().await;
        
        eprintln!("{:?}", res);
        
    });
    
}