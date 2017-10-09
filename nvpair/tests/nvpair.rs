extern crate nvpair;
use std::ffi::CStr;

#[test]
fn new()
{
    let a = nvpair::NvList::new().unwrap();
    assert!(a.is_empty());
}

#[test]
fn size_empty_native()
{
    let a = nvpair::NvList::new().unwrap();
    assert_eq!(a.encoded_size(nvpair::NvEncoding::Native).unwrap(), 16);
}

#[test]
fn add_boolean()
{
    let mut a = nvpair::NvList::new().unwrap();
    a.add_boolean(&b"hi\0"[..]).unwrap();
    assert_eq!(a.encoded_size(nvpair::NvEncoding::Native).unwrap(), 40);
    let p = a.first().unwrap();
    assert_eq!(p.name(), CStr::from_bytes_with_nul(&b"hi\0"[..]).unwrap());

    assert!(a.exists(&b"hi\0"[..]));
}
