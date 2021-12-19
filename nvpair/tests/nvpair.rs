extern crate nvpair;
use std::ffi::{CStr, CString};

#[test]
fn new() {
    let a = nvpair::NvList::new();
    assert!(a.is_empty());
}

#[test]
fn size_empty_native() {
    let a = nvpair::NvList::new();
    assert_eq!(a.encoded_size(nvpair::NvEncoding::Native).unwrap(), 16);
}

#[test]
fn add_boolean() {
    let mut a = nvpair::NvList::new();
    a.add_boolean(&b"hi\0"[..]).unwrap();
    assert_eq!(a.encoded_size(nvpair::NvEncoding::Native).unwrap(), 40);
    let p = a.first().unwrap();
    assert_eq!(p.name(), CStr::from_bytes_with_nul(&b"hi\0"[..]).unwrap());

    assert!(a.exists(&b"hi\0"[..]));
    assert!(!a.exists("bye"));
}

#[test]
fn iter() {
    let ns = ["one", "two", "three"];
    let mut a = nvpair::NvList::new();

    for n in ns.iter() {
        a.add_boolean(*n).unwrap();
    }

    let mut ct = 0;
    for i in a.iter().zip(ns.iter()) {
        ct += 1;
        let a = i.0.name();
        let b = CString::new(*i.1).unwrap();
        assert_eq!(a, b.as_c_str());
    }

    assert_eq!(ct, ns.len());
}

#[test]
fn lookup() {
    let ns = ["one", "two", "three"];
    let mut a = nvpair::NvList::new_unique_names();

    for n in ns.iter() {
        a.add_boolean(*n).unwrap();
    }

    a.lookup("one").unwrap();
}

#[test]
fn insert() {
    let mut a = nvpair::NvList::new_unique_names();
    a.insert("bool1", &true).unwrap();
    a.insert("u32", &6u32).unwrap();

    let b1 = a.lookup("bool1").expect("lookup of bool1 failed");

    match b1.data() {
        nvpair::NvData::BoolV(v) => {
            assert!(v == true);
        }
        _ => {
            panic!("Unexpected type");
        }
    }

    let u1 = a.lookup("u32").expect("lookup of u32 failed");

    match u1.data() {
        nvpair::NvData::Uint32(v) => {
            assert!(v == 6u32);
        }
        _ => {
            panic!("Unexpected type");
        }
    }

    //a.remove(&b1).expect("remove of b1 failed");

    // FIXME: use option wrapper
    //assert!(a.lookup("bool1").is_err());
}

#[test]
fn insert_cstr() {
    let mut a = nvpair::NvList::new();

    a.insert("hello", CStr::from_bytes_with_nul(b"bye\0").unwrap())
        .unwrap();
}

#[test]
fn lookup_nvlist() {
    let mut a = nvpair::NvList::new();
    let mut b = nvpair::NvList::new();

    a.insert("foo", "bar").unwrap();
    b.insert("baz", a.as_ref()).unwrap();

    {
        let nvlr: &nvpair::NvListRef = b.lookup_nvlist("baz").unwrap();
        nvlr.lookup_string("foo").unwrap();
    }

    {
        let nvlr: &nvpair::NvListRef = b.lookup_nvlist("baz").unwrap();
        nvlr.lookup_string("foo").unwrap();
    }
}
