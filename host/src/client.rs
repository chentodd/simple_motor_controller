use std::convert::Infallible;

use postcard_rpc::{
    header::VarSeqKind,
    host_client::{HostClient, HostErr},
    standard_icd::{ERROR_PATH, PingEndpoint, WireError},
};

use protocol::*;

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

trait FlattenErr {
    type Good;
    type Bad;
    fn flatten(self) -> Result<Self::Good, ClientError<Self::Bad>>;
}

impl<T, E> FlattenErr for Result<T, E> {
    type Good = T;
    type Bad = E;

    fn flatten(self) -> Result<Self::Good, ClientError<Self::Bad>> {
        self.map_err(|e| ClientError::Endpoint(e))
    }
}

// ---

impl Client {
    pub fn new(product_name: &str) -> Result<Self, String> {
        let client = HostClient::try_new_raw_nusb(
            |d| d.product_string() == Some(product_name),
            ERROR_PATH,
            8,
            VarSeqKind::Seq2,
        )?;
        Ok(Self { client })
    }

    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    pub async fn ping(&self, id: u32) -> Result<u32, ClientError<Infallible>> {
        let val = self.client.send_resp::<PingEndpoint>(&id).await?;
        Ok(val)
    }

    pub async fn set_motor_cmd(
        &self,
        id: MotorId,
        cmd: MotorCommand,
    ) -> Result<(), ClientError<CommandError>> {
        self.client
            .send_resp::<SetMotorCommandEndPoint>(&(id, cmd))
            .await?
            .flatten()
    }

    pub async fn set_motor_cmds(
        &self,
        cmds: [(MotorId, MotorCommand); 2],
    ) -> Result<(), ClientError<CommandError>> {
        self.client
            .send_resp::<SetMotorCommandsEndPoint>(&cmds)
            .await?
            .flatten()
    }
}
