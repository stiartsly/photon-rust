#[macro_export]
macro_rules! as_millis {
    ($time:expr) => {{
        $time.elapsed().unwrap().as_millis()
    }};
}

#[macro_export]
macro_rules! as_usize {
    ($val:expr) => {{
        $val as usize
    }};
}

#[macro_export]
macro_rules! as_uchar_ptr {
    ($val:expr) => {{
        $val.as_ptr() as *const libc::c_uchar
    }};
}

#[macro_export]
macro_rules! as_uchar_ptr_mut {
    ($val:expr) => {{
        $val.as_mut_ptr() as *mut libc::c_uchar
    }};
}

#[macro_export]
macro_rules! unwrap {
    ($val:expr) => {{
        $val.as_ref().unwrap()
    }};
}

#[macro_export]
macro_rules! as_kind_name {
    ($val:expr) => {{
        match $val.is_ipv4() {
            true => "ipv4",
            false => "ipv6"
        }
    }};
}
