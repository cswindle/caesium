use std;
use nom;
use nom::{le_u32};

use errors::*;

named!(crate_parser<&[u8], (&str, &[u8])>,
  do_parse!(
    manifest_len: le_u32           >>
    manifest: map_res!(
        take!(manifest_len),
        std::str::from_utf8
    )                              >>
    tar_len: le_u32                >>
    tar: take!(tar_len)            >>

    (manifest, tar)
  )
);

pub fn parse_crate_upload(upload: &[u8]) -> Result<(&str, &[u8])> {
    match crate_parser(upload) {
        nom::IResult::Done(_,(manifest, tar)) => Ok((manifest, tar)),
        _ => bail!("Failed to parse binary"),
    }
}
