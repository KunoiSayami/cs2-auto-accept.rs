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

#[cfg(not(feature = "gui"))]
pub(crate) mod gui {
    #[macro_export]
    macro_rules! update_status {
        ($($arg:tt)*) => {};
        ($x:expr, $y: expr) => {};
    }

    pub(crate) fn gui_entry(config: &String, force_distance: bool) -> anyhow::Result<()> {
        crate::real_main_guarder(config, force_distance)
    }
}
