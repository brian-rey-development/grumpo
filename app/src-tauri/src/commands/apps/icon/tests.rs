use super::*;

#[test]
fn encodes_a_png_as_a_data_url() {
    let png = [0x89, b'P', b'N', b'G'];
    assert_eq!(data_url(&png), "data:image/png;base64,iVBORw==");
}

#[test]
fn an_empty_image_still_has_the_prefix() {
    assert_eq!(data_url(&[]), "data:image/png;base64,");
}
