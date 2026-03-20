use clap::builder::styling::AnsiColor;
use clap::builder::Styles;

pub fn custom_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_styles_returns_styles() {
        // Simply verify that `custom_styles` can be called without panicking and
        // returns a `Styles` value (the type is non-exhaustive so we can't inspect
        // individual fields, but we can verify the function works end-to-end).
        let _styles = custom_styles();
    }
}
