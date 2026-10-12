#![warn(rust_2018_idioms)]

use bytes::{Buf, Bytes, BytesMut};

#[test]
fn static_and_owner_have_no_hint() {
    assert_eq!(Bytes::new().allocation_size_hint(), None);
    assert_eq!(Bytes::from_static(b"hello").allocation_size_hint(), None);
    assert_eq!(
        Bytes::from_owner(vec![1u8; 10]).allocation_size_hint(),
        None
    );
}

#[test]
fn promotable_reports_boxed_slice() {
    // len == cap, so this is the promotable (boxed slice) representation.
    let mut b = Bytes::from(vec![7u8; 100]);
    assert_eq!(b.allocation_size_hint(), Some(100));

    // Advancing an unpromoted handle keeps the whole allocation.
    b.advance(30);
    assert_eq!(b.len(), 70);
    assert_eq!(b.allocation_size_hint(), Some(100));

    // Slicing promotes it to shared; both handles still report the whole allocation.
    let c = b.slice(10..20);
    assert_eq!(b.allocation_size_hint(), Some(100));
    assert_eq!(c.allocation_size_hint(), Some(100));
}

#[test]
fn vec_with_spare_capacity_reports_full_capacity() {
    let mut v = Vec::with_capacity(4096);
    v.extend_from_slice(&[1u8; 10]);
    let b = Bytes::from(v);
    assert_eq!(b.allocation_size_hint(), Some(4096));
    assert_eq!(b.slice(2..4).allocation_size_hint(), Some(4096));
}

#[test]
fn bytes_mut_chunks_report_shared_buffer() {
    let mut buf = BytesMut::with_capacity(64 * 1024);
    buf.extend_from_slice(&[0u8; 40 * 1024]);

    let first = buf.split_to(16 * 1024).freeze();
    let second = buf.split_to(16 * 1024).freeze();
    assert_eq!(first.allocation_size_hint(), Some(64 * 1024));
    assert_eq!(second.allocation_size_hint(), Some(64 * 1024));

    // Reserving while chunks are outstanding moves `buf` to a new allocation; the
    // outstanding chunks still report (and pin) the old one.
    buf.reserve(128 * 1024);
    buf.extend_from_slice(&[0u8; 10]);
    let third = buf.split().freeze();
    assert!(third.allocation_size_hint().unwrap() >= 128 * 1024);
    assert_eq!(first.allocation_size_hint(), Some(64 * 1024));
}

#[test]
fn unshared_bytes_mut_freeze() {
    let mut buf = BytesMut::with_capacity(1000);
    buf.extend_from_slice(&[0u8; 10]);
    assert_eq!(buf.freeze().allocation_size_hint(), Some(1000));
}
