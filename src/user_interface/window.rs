use dioxus::prelude::*;

#[derive(Copy, Clone)]
pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

pub(super) fn use_window_size_provider() -> Signal<WindowSize> {
    use_context_provider(|| Signal::new(get_window_size()))
}

pub fn use_window_size() -> ReadOnlySignal<WindowSize> {
    let signal = use_context::<Signal<WindowSize>>();
    ReadOnlySignal::new(signal)
}

impl WindowSize {
    pub fn is_widescreen(&self) -> bool {
        let WindowSize { width, height } = self;
        width * 3. >= height * 4.
    }
}

#[cfg(any(feature = "web", feature = "desktop"))]
fn get_window_size() -> WindowSize {
    #[cfg(feature = "web")]
    {
        return WindowSize {
            width: web_sys::window()
                .map(|window| window.inner_width().ok().map(|width| width.as_f64()))
                .flatten()
                .flatten()
                .unwrap_or_default(),
            height: web_sys::window()
                .map(|window| window.inner_height().ok().map(|height| height.as_f64()))
                .flatten()
                .flatten()
                .unwrap_or_default(),
        };
    }

    #[cfg(feature = "desktop")]
    {
        use dioxus_desktop::DesktopContext;
        let inner_size = consume_context::<DesktopContext>().window.inner_size();
        return WindowSize {
            width: inner_size.width as f64,
            height: inner_size.height as f64,
        };
    }

    return WindowSize {
        width: 800.0,
        height: 600.0,
    };
}
