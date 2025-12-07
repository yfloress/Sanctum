slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let ui = AppWindow::new()?;

    println!("Sanctum Core Initialized.");
    ui.run()?;

    Ok(())
}
