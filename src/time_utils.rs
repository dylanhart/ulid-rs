pub(crate) fn now() -> std::time::SystemTime {
    #[cfg(target_arch = "wasm32")]
    {
        use web_time::web::SystemTimeExt;
        return web_time::SystemTime::now().to_std();
    }
    #[cfg(not(target_arch = "wasm32"))]
    return std::time::SystemTime::now();
}
