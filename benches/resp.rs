use bytes::BytesMut;
use criterion::{criterion_group, criterion_main, Criterion};
use rredis::{parse_frame, parse_frame_length, RespFrame};
use std::hint::black_box;

const DATA: &str = "+OK\r\n-ERR\r\n:1000\r\n$6\r\nfoobar\r\n$-1\r\n*2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n";

/**
1_decode                time:   [2.8112 µs 2.8356 µs 2.8726 µs]
                        change: [-1.2894% +0.4318% +2.0285%] (p = 0.63 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

v2_decode               time:   [2.3297 µs 2.3472 µs 2.3767 µs]
                        change: [+0.3845% +1.1785% +2.1590%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
 */
fn v1_decode(buf: &mut BytesMut) -> anyhow::Result<Vec<RespFrame>> {
    use rredis::RespDecode;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode(buf: &mut BytesMut) -> anyhow::Result<Vec<RespFrame>> {
    use rredis::RespDecodeV2;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode_no_buf_clone(buf: &mut &[u8]) -> anyhow::Result<Vec<RespFrame>> {
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let _len = parse_frame_length(buf)?;
        let frame = parse_frame(buf).unwrap();
        frames.push(frame)
    }
    Ok(frames)
}

fn v2_decode_parse_length(buf: &mut &[u8]) -> anyhow::Result<()> {
    use rredis::RespDecodeV2;
    while !buf.is_empty() {
        let len = RespFrame::expect_length(buf)?;
        *buf = &buf[len..];
    }
    Ok(())
}

fn v1_decode_parse_length(buf: &mut &[u8]) -> anyhow::Result<()> {
    use rredis::RespDecode;
    while !buf.is_empty() {
        let len = RespFrame::expect_length(buf)?;
        *buf = &buf[len..];
    }
    Ok(())
}

fn v2_decode_parse_frame(buf: &mut &[u8]) -> anyhow::Result<Vec<RespFrame>> {
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = parse_frame(buf).unwrap();
        frames.push(frame);
    }
    Ok(frames)
}

fn criterion_benchmark(c: &mut Criterion) {
    let buf = BytesMut::from(DATA);

    c.bench_function("v1_decode", |b| {
        b.iter(|| v1_decode(black_box(&mut buf.clone())))
    });

    c.bench_function("v2_decode", |b| {
        b.iter(|| v2_decode(black_box(&mut buf.clone())))
    });

    c.bench_function("v2_decode_no_buf_clone", |b| {
        b.iter(|| v2_decode_no_buf_clone(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("v1_decode_parse_length", |b| {
        b.iter(|| v1_decode_parse_length(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("v2_decode_parse_length", |b| {
        b.iter(|| v2_decode_parse_length(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("v2_decode_parse_frame", |b| {
        b.iter(|| v2_decode_parse_frame(black_box(&mut DATA.as_bytes())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
