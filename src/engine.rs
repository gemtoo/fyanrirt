use crate::args;
use crate::misc::*;
use colored::Colorize;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use rusmpp::commands::types::DataCoding;
use rusmpp::{
    commands::{
        tlvs::tlv::message_submission_request::MessageSubmissionRequestTLVValue,
        types::{EsmClass, InterfaceVersion, Npi, RegisteredDelivery, ServiceType, Ton},
    },
    pdu::{self, Bind, SubmitSm},
    types::{AnyOctetString, COctetString, OctetString},
    Command, CommandCodec, CommandId, CommandStatus, Pdu, TLVTag,
};
use std::error::Error;
use std::str::FromStr;

use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};

use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tracing::{span, Instrument, Level};

#[derive(Debug, Clone)]
pub struct SMPPCredentials {
    pub provider_name: String,
    pub endpoint_addr: String,
    pub system_id: COctetString<1, 16>,
    pub password: COctetString<1, 9>,
    pub system_type: COctetString<1, 13>,
    // pub rate_limit: u64,
    // pub smppv
}
use uuid::Uuid;
pub struct SMS {
    pub src: String,
    pub dst: String,
    pub content: String,
}
impl SMS {
    fn from(src: impl Into<String>, dst: impl Into<String>, content: impl Into<String>) -> Self {
        SMS {
            src: src.into(),
            dst: dst.into(),
            content: content.into(),
        }
    }
}

struct SMPPTransmitter(FramedWrite<OwnedWriteHalf, CommandCodec>);

impl SMPPTransmitter {
    async fn command_bind_transceiver(
        &mut self,
        credentials: SMPPCredentials,
    ) -> Result<(), Box<dyn Error>> {
        // Authentication
        let tx = &mut self.0;
        trace!("Building transceiver binding command ...");
        let bind_transceiver_command = Command::new(
            CommandStatus::EsmeRok,
            1,
            Bind::builder()
                .system_id(credentials.system_id)
                .password(credentials.password)
                .system_type(credentials.system_type)
                .interface_version(InterfaceVersion::Smpp3_4)
                .addr_ton(Ton::Unknown)
                .addr_npi(Npi::Unknown)
                .address_range(COctetString::empty())
                .build()
                .into_bind_transceiver(),
        );
        trace!("Writing transceiver binding command to TCP endpoint ...");
        tx.send(&bind_transceiver_command).await?;
        Ok(())
    }
    async fn command_submit_sm(&mut self, sms: SMS) {
        let tx = &mut self.0;
        trace!(
            "Building SMS submit command. Source: '{}'. Destination: '{}'. Content: '{}'.",
            &sms.src.cyan(),
            &sms.dst.cyan(),
            &sms.content.cyan()
        );
        let encodings: Vec<DataCoding> = vec![
            DataCoding::McSpesific,
            DataCoding::Ia5,
            DataCoding::OctetUnspecified,
            DataCoding::Latin1,
            DataCoding::OctetUnspecified2,
            DataCoding::Jis,
            DataCoding::Cyrillic,
            DataCoding::LatinHebrew,
            DataCoding::Ucs2,
            DataCoding::PictogramEncoding,
            DataCoding::Iso2022JpMusicCodes,
            DataCoding::ExtendedKanjiJis,
            DataCoding::Ksc5601,
            DataCoding::GsmMwiControl,
            DataCoding::GsmMwiControl2,
            DataCoding::GsmMessageClassControl,
            DataCoding::Other(72),
        ];
        //for encoding in encodings {
            // let numb: u8 = encoding.try_into().unwrap();
            // let cyancoding = format!("{:?}", encoding).cyan();
            // trace!("Using data coding {}, a.k.a. {}.", numb.to_string().cyan(), cyancoding);
            let submit_sm_command = Command::new(
                CommandStatus::EsmeRok,
                2,
                SubmitSm::builder()
                    .serivce_type(ServiceType::default())
                    .source_addr_ton(Ton::Unknown)
                    .source_addr_npi(Npi::Unknown)
                    .source_addr(COctetString::from_str(&sms.src).unwrap())
                    .destination_addr(COctetString::from_str(&sms.dst).unwrap())
                    .esm_class(EsmClass::default())
                    .registered_delivery(RegisteredDelivery::request_all())
                    .short_message(OctetString::from_str(&sms.content).unwrap())
                    .data_coding(DataCoding::Ucs2)
                    /*.push_tlv(
                        MessageSubmissionRequestTLVValue::MessagePayload(AnyOctetString::from_str(
                            tag_length_value,
                        )?)
                        .into(),
                    )*/
                    .build()
                    .into_submit_sm(),
            );
            trace!("Writing SMS submit command to TCP endpoint ...");
            tx.send(&submit_sm_command).await.unwrap();
            sleep(10000).await;
        //}
    }
    async fn command_unbind(&mut self) -> Result<(), Box<dyn Error>> {
        let tx = &mut self.0;
        debug!("Building unbinding command ...");
        let unbind_command = Command::new(CommandStatus::EsmeRok, 3, Pdu::Unbind);
        trace!("Writing unbinding command to TCP endpoint ...");
        tx.send(&unbind_command).await?;
        Ok(())
    }
}
struct SMPPReceiver(FramedRead<OwnedReadHalf, CommandCodec>);
impl SMPPReceiver {
    async fn subscribe_bind_transceiver(&mut self) {
        let rx = &mut self.0;
        trace!("Waiting for response from TCP endpoint ...");
        while let Some(result) = rx.next().await {
            match result {
                Ok(command) => match command.pdu() {
                    Some(Pdu::BindTransceiverResp(response)) => {
                        trace!(
                            "Transceiver response received: '{}'.",
                            response.system_id.to_str().unwrap().cyan()
                        );

                        if let CommandStatus::EsmeRok = command.command_status {
                            info!("Bound transceiver successfully.");
                            break;
                        } else {
                            die("Command status error.");
                        }
                    }
                    _ => {
                        dbg!(command);
                        die("Unexpected PDU. Most likely wrong credentials.");
                    }
                },
                Err(e) => {
                    let reason = format!("Error receiving command: {:?}", e);
                    die(reason);
                }
            }
        }
    }
    async fn subscribe_reciepts(&mut self) {
        let rx = &mut self.0;
        loop {
            let frame = rx.next().await;
            match frame {
                Some(command_result) => {
                    match command_result {
                        Ok(command) => {
                            match command.pdu() {
                                Some(Pdu::SubmitSmResp(response)) => {
                                    info!(
                                        "Submit SMS response received: '{}'.",
                                        response.message_id().to_str().unwrap().cyan()
                                    );

                                    if let CommandStatus::EsmeRok = command.command_status {
                                        info!(
                                            "SMS submitted successfully: {:?}.",
                                            command.command_status
                                        );
                                    } else {
                                        error!("here");
                                    }
                                }
                                Some(Pdu::DeliverSm(deliver_sm)) => {
                                    info!(
                                        "DeliverSM received.",
                                        //deliver_sm.short_message().to_str().unwrap().cyan()
                                    );
                                    dbg!(deliver_sm);
                                    // for tlv in deliver_sm.tlvs().iter() {
                                    //     trace!("Received {:?}.", tlv);
                                    // }
                                }
                                Some(Pdu::Unbind) => {
                                    trace!("Received unbind PDU.");
                                    //tx.unbind();
                                    todo!();
                                }
                                Some(pdu) => {
                                    trace!("Received PDU: {:?}.", pdu);
                                    // if pdu unbind then drop connection
                                }
                                None => {
                                    debug!("PDU is None.");
                                    dbg!(command);
                                    todo!();
                                }
                            }
                        }
                        Err(e) => {
                            error!("{}", e);
                            dbg!(e);
                        }
                    }
                }
                None => {
                    info!("End of TCP stream.");
                    break;
                }
            }
        }
    }
    async fn subscribe_unbind(&mut self) {
        let rx = &mut self.0;
        trace!("Waiting for unbinding response ...");
        while let Some(Ok(command)) = rx.next().await {
            if let CommandId::UnbindResp = command.command_id() {
                trace!("Unbinding response received.");
                if let CommandStatus::EsmeRok = command.command_status {
                    info!("Unbound successfully: {:?}.", command.command_status);
                    break;
                }
            }
        }
    }
}
struct SMPPConnection {
    rx: SMPPReceiver,
    tx: SMPPTransmitter,
}

impl SMPPConnection {
    async fn from(credentials: SMPPCredentials) -> Result<Self, Box<dyn Error>> {
        // Initialization of TCP connection
        debug!(
            "Establishing TCP connection to {} ...",
            credentials.endpoint_addr.green()
        );
        let stream = TcpStream::connect(&credentials.endpoint_addr).await?;
        let (reader, writer) = stream.into_split();
        let mut rx = SMPPReceiver(FramedRead::new(reader, CommandCodec {}));
        let mut tx = SMPPTransmitter(FramedWrite::new(writer, CommandCodec {}));
        trace!(
            "TCP connection with provider '{}' has been established.",
            credentials.provider_name.cyan()
        );
        tx.command_bind_transceiver(credentials).await.unwrap();
        rx.subscribe_bind_transceiver().await;
        let connection = SMPPConnection { rx, tx };
        Ok(connection)
    }
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let credentials = args::credentials().unwrap();
    let sms = args::smsoneshot();
    // let rate_limit = credentials.rate_limit;
    // Tracing bolerplate
    let session_id = Uuid::new_v4().to_string();
    let smpp_span = span!(Level::TRACE, "smpp-client", id = session_id);
    let _ = smpp_span.enter();

    let smscconn = SMPPConnection::from(credentials).await.unwrap();

    let tx_handle = tokio::spawn(
        async move {
            debug!("Spawning TX thread ...");
            let mut tx = smscconn.tx;
            tx.command_submit_sm(sms).await;
            // sleep is done not to drop TCP connection.
            sleep(20000).await;
            tx.command_unbind().await;
        }
        .instrument(smpp_span.clone()),
    );

    let rx_handle = tokio::spawn(
        async move {
            debug!("Spawning RX thread ...");
            let mut rx = smscconn.rx;
            rx.subscribe_reciepts().await;
        }
        .instrument(smpp_span.clone()),
    );
    tokio::join!(tx_handle, rx_handle);
    Ok(())
}
