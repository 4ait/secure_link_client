use tokio::runtime::Runtime;
use secure_link_client::SecureLink;

fn main() {

    let auth_token = "1:44qtO_YnOAu3JzgG-BmmPtWKibRGbSIi3E6P2iZ0wFs";

    let secure_link_server_host = "192.168.12.16";
    let secure_link_server_port: u16 = 6001;

    Runtime::new().unwrap().block_on(async {

        let secure_link_connection_result =
            SecureLink::connect_to_global_channel(
                secure_link_server_host,
                secure_link_server_port,
                &auth_token
            ).await.unwrap();
        
        let res = secure_link_connection_result.run_message_loop().await;
        
        
        println!("{:?}", res);


    });
    
}