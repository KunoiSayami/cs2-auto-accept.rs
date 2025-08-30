#[cfg(not(feature = "distance"))]
pub(crate) mod distance {
    pub(crate) fn calc_color_distance(
        _: clap::parser::ValuesRef<'_, String>,
        _: &String,
        _: bool,
        _: bool,
    ) -> ! {
        unimplemented!("To use this function, enable \"distance\" feature")
    }
}

#[cfg(not(feature = "jpeg"))]
pub(crate) mod dir_match {
    pub(crate) fn test_files(_: &str, _: &str, _: bool) -> ! {
        unimplemented!("To use this function, enable \"jpeg\" feature")
    }
}
