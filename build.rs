use std::{env, io};
use winresource::WindowsResource;

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("./assets/backstop_icon.ico")
            .set("ProductName", "Backstop")
            .set("FileDescription", "Backstop")
            .compile()?;
    }

    Ok(())
}
