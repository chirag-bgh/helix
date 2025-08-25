use std::{io::Read, str::FromStr, sync::Arc};

use axum::http::{header, Method, Request, Uri};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use flate2::read::GzDecoder;
use helix_types::{BidTrace, BlobsBundle, BlsSignature, ExecutionPayloadDeneb};
use serde::{Deserialize, Serialize};
use ssz::Decode;
use ssz_derive::{Decode, Encode};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(deny_unknown_fields)]
pub struct SignedBidSubmissionDeneb {
    pub message: BidTrace,
    pub execution_payload: Arc<ExecutionPayloadDeneb>,
    pub blobs_bundle: Arc<BlobsBundle>,
    pub signature: BlsSignature,
}

pub fn generate_request(
    cancellations_enabled: bool,
    gzip_encoding: bool,
    ssz_content_type: bool,
    payload: &[u8],
) -> Request<axum::body::Body> {
    let uri_str = if cancellations_enabled {
        "http://example.com?cancellations=1"
    } else {
        "http://example.com"
    };
    let uri = Uri::from_str(uri_str).unwrap();

    let method = Method::POST;
    let body = axum::body::Body::from(payload.to_vec());
    let mut request_builder = Request::builder().method(method).uri(uri);

    if gzip_encoding {
        request_builder = request_builder.header(header::CONTENT_ENCODING, "gzip");
    }
    if ssz_content_type {
        request_builder = request_builder.header(header::CONTENT_TYPE, "application/octet-stream");
    } else {
        request_builder = request_builder.header(header::CONTENT_TYPE, "application/json");
    }
    request_builder.body(body).unwrap()
}

fn decompress_gzip(compressed_data: &[u8]) -> Vec<u8> {
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

pub fn benchmark_ssz_decode(c: &mut Criterion) {
    let req_payload_bytes_ssz_gz =
        include_bytes!("../test_data/submitBlockPayloadDeneb_Goerli.ssz.gz");
    let ssz_bytes = decompress_gzip(req_payload_bytes_ssz_gz);
    println!(
        "SSZ size: {} bytes ({:.2} MB)",
        ssz_bytes.len(),
        ssz_bytes.len() as f64 / 1024.0 / 1024.0
    );

    let mut group = c.benchmark_group("payload_decoding");

    group.bench_with_input(
        BenchmarkId::new("ethereum_ssz with lh types", "deneb_payload"),
        &ssz_bytes.as_slice(),
        |b, payload| {
            b.iter(|| {
                let submission =
                    black_box(SignedBidSubmissionDeneb::from_ssz_bytes(payload).unwrap());
                black_box(submission)
            });
        },
    );

    group.finish();
}

criterion_group!(benches, benchmark_ssz_decode);
criterion_main!(benches);
