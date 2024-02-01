pub(crate) fn now() -> std::time::SystemTime {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        use web_time::web::SystemTimeExt;
        return web_time::SystemTime::now().to_std();
    }
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    return std::time::SystemTime::now();
}
