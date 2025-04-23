use std::convert::Infallible;

use postcard_rpc::{
    header::VarSeqKind,
    host_client::{HostClient, HostErr},
    standard_icd::{ERROR_PATH, PingEndpoint, WireError},
};

// use protocol::*;

pub struct Client {
    pub client: HostClient<WireError>,
}

#[derive(Debug)]
pub enum ClientError<E> {
    Comms(HostErr<WireError>),
    Endpoint(E),
}

impl<E> From<HostErr<WireError>> for ClientError<E> {
    fn from(err: HostErr<WireError>) -> Self {
        ClientError::Comms(err)
    }
}

// trait FlattenErr {
//     type Good;
//     type Bad;
//     fn flatten(self) -> Result<Self::Good, ClientError<Self::Bad>>;
// }

// impl<T, E> FlattenErr for Result<T, E> {
//     type Good = T;
//     type Bad = E;

//     fn flatten(self) -> Result<Self::Good, ClientError<Self::Bad>> {
//         self.map_err(|e| ClientError::Endpoint(e))
//     }
// }

// ---

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        let client = HostClient::new_raw_nusb(
            |d| d.product_string() == Some("stm32-discovery"),
            ERROR_PATH,
            8,
            VarSeqKind::Seq2,
        );
        println!("Client created");
        Self { client }
    }

    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    pub async fn ping(&self, id: u32) -> Result<u32, ClientError<Infallible>> {
        let val = self.client.send_resp::<PingEndpoint>(&id).await?;
        Ok(val)
    }
}
