extern crate nvpair;

#[test]
fn new()
{
    let _ = nvpair::NvList::new().unwrap();
}

#[test]
fn size_empty_native()
{
    let a = nvpair::NvList::new().unwrap();
    assert_eq!(a.encoded_size(nvpair::NvEncoding::Native).unwrap(), 16);
}
