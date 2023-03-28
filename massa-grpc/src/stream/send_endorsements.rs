use crate::error::{match_for_io_error, GrpcError};
use crate::service::MassaGrpcService;
use futures_util::StreamExt;
use massa_models::endorsement::{EndorsementDeserializer, SecureShareEndorsement};
use massa_models::secure_share::SecureShareDeserializer;
use massa_proto::massa::api::v1::{self as grpc};
use massa_serialization::{DeserializeError, Deserializer};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::pin::Pin;
use tonic::codegen::futures_core;
use tracing::log::{error, warn};

/// Type declaration for SendEndorsementsStream
pub type SendEndorsementsStream = Pin<
    Box<
        dyn futures_core::Stream<Item = Result<grpc::SendEndorsementsStreamResponse, tonic::Status>>
            + Send
            + 'static,
    >,
>;

/// Send endorsements
pub(crate) async fn send_endorsements(
    grpc: &MassaGrpcService,
    request: tonic::Request<tonic::Streaming<grpc::SendEndorsementsStreamRequest>>,
) -> Result<SendEndorsementsStream, GrpcError> {
    let mut pool_command_sender = grpc.pool_command_sender.clone();
    let mut protocol_command_sender = grpc.protocol_command_sender.clone();
    let config = grpc.grpc_config.clone();
    let storage = grpc.storage.clone_without_refs();

    let (tx, rx) = tokio::sync::mpsc::channel(config.max_channel_size);
    let mut in_stream = request.into_inner();

    tokio::spawn(async move {
        while let Some(result) = in_stream.next().await {
            match result {
                Ok(req_content) => {
                    if req_content.endorsements.is_empty() {
                        report_error(
                            req_content.id.clone(),
                            tx.clone(),
                            tonic::Code::InvalidArgument,
                            "the request payload is empty".to_owned(),
                        )
                        .await;
                    } else {
                        let proto_endorsement = req_content.endorsements;
                        if proto_endorsement.len() as u32 > config.max_endorsements_per_message {
                            report_error(
                                req_content.id.clone(),
                                tx.clone(),
                                tonic::Code::InvalidArgument,
                                "too many endorsements".to_owned(),
                            )
                            .await;
                        } else {
                            let endorsement_deserializer =
                                SecureShareDeserializer::new(EndorsementDeserializer::new(
                                    config.thread_count,
                                    config.endorsement_count,
                                ));
                            let verified_eds_res: Result<HashMap<String, SecureShareEndorsement>, GrpcError> = proto_endorsement
                                .into_iter()
                                .map(|proto_endorsement| {
                                    let mut ed_serialized = Vec::new();
                                    ed_serialized.extend(proto_endorsement.signature.as_bytes());
                                    ed_serialized.extend(proto_endorsement.content_creator_pub_key.as_bytes());
                                    ed_serialized.extend(proto_endorsement.serialized_data);

                                    let verified_op = match endorsement_deserializer.deserialize::<DeserializeError>(&ed_serialized) {
                                        Ok(tuple) => {
                                            let (rest, res_endorsement): (&[u8], SecureShareEndorsement) = tuple;
                                            if rest.is_empty() {
                                                res_endorsement.verify_signature()
                                                    .map(|_| (res_endorsement.id.to_string(), res_endorsement))
                                                    .map_err(|e| e.into())
                                            } else {
                                                Err(GrpcError::InternalServerError(
                                                    "there is data left after endorsement deserialization".to_owned()
                                                ))
                                            }
                                        }
                                        Err(e) => {
                                            Err(GrpcError::InternalServerError(format!("failed to deserialize endorsement: {}", e)
                                            ))
                                        }
                                    };
                                    verified_op
                                })
                                .collect();

                            match verified_eds_res {
                                Ok(verified_eds) => {
                                    let mut endorsement_storage = storage.clone_without_refs();
                                    endorsement_storage.store_endorsements(
                                        verified_eds.values().cloned().collect(),
                                    );
                                    pool_command_sender.add_endorsements(endorsement_storage.clone());

                                    if let Err(e) =
                                        protocol_command_sender.propagate_endorsements(endorsement_storage)
                                    {
                                        let error =
                                            format!("failed to propagate endorsement: {}", e);
                                        report_error(
                                            req_content.id.clone(),
                                            tx.clone(),
                                            tonic::Code::Internal,
                                            error.to_owned(),
                                        )
                                        .await;
                                    };

                                    let result = grpc::EndorsementResult {
                                        endorsements_ids: verified_eds.keys().cloned().collect(),
                                    };
                                    if let Err(e) = tx
                                        .send(Ok(grpc::SendEndorsementsStreamResponse {
                                            id: req_content.id.clone(),
                                            message: Some(
                                                grpc::send_endorsements_stream_response::Message::Result(
                                                    result,
                                                ),
                                            ),
                                        }))
                                        .await
                                    {
                                        error!("failed to send back endorsement response: {}", e)
                                    };
                                }
                                Err(e) => {
                                    let error = format!("invalid endorsement(s): {}", e);
                                    report_error(
                                        req_content.id.clone(),
                                        tx.clone(),
                                        tonic::Code::InvalidArgument,
                                        error.to_owned(),
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    if let Some(io_err) = match_for_io_error(&err) {
                        if io_err.kind() == ErrorKind::BrokenPipe {
                            warn!("client disconnected, broken pipe: {}", io_err);
                            break;
                        }
                    }
                    error!("{}", err);
                    if let Err(e) = tx.send(Err(err)).await {
                        error!(
                            "failed to send back send_endorsements error response: {}",
                            e
                        );
                        break;
                    }
                }
            }
        }
    });

    let out_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Ok(Box::pin(out_stream) as SendEndorsementsStream)
}

/// This function reports an error to the sender by sending a gRPC response message to the client
async fn report_error(
    id: String,
    sender: tokio::sync::mpsc::Sender<Result<grpc::SendEndorsementsStreamResponse, tonic::Status>>,
    code: tonic::Code,
    error: String,
) {
    error!("{}", error);
    // Attempt to send the error response message to the sender
    if let Err(e) = sender
        .send(Ok(grpc::SendEndorsementsStreamResponse {
            id,
            message: Some(grpc::send_endorsements_stream_response::Message::Error(
                massa_proto::google::rpc::Status {
                    code: code.into(),
                    message: error,
                    details: Vec::new(),
                },
            )),
        }))
        .await
    {
        // If sending the message fails, log the error message
        error!(
            "failed to send back send_endorsements error response: {}",
            e
        );
    }
}
