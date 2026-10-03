/*!
AFRA (ARM Flow Reconstruction and Analysis) is a control flow graph reconstructor for AArch64 binaries in basic block level.
!*/

use afra::module::ui::App;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let mut app = App::new();

    let result = app.run(terminal);

    ratatui::restore();

    result
}
