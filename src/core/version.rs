use std::string::String;

pub(crate) const NODE_TAG_NAME: &str = "MK";
pub(crate) const NODE_VERSION: i32 = 1;

pub(crate) fn build(_: &str, _: i32) -> i32 {
    unimplemented!()
}

pub(crate) fn formatted_version(_: i32) -> String {
   // unimplemented!()
   return "1".to_string();
}
