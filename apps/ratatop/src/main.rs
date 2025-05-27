use app::App;

pub mod app;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let app = App::new();

    let result = app.run(terminal);

    ratatui::restore();

    result
}
