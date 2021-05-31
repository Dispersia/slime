use crate::app::{App, AppSettings};

#[cfg(target_arch = "wasm32")]
pub fn run_app<F>(settings: AppSettings, runner: F)
where
    F: 'static + FnOnce(App),
{
    wasm_bindgen_futures::spawn_local(async move {
        let app = App::new(settings).await;
        runner(app);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_app<F>(settings: AppSettings, runner: F)
where
    F: 'static + FnOnce(App),
{
    let app = pollster::block_on(App::new(settings));
    runner(app);
}
