use std::{
    fmt::Arguments,
    sync::{Mutex, Once},
};

use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch, FormatCallback, Output,
};
use log::{LevelFilter, Log, Record};

fn set_dispatch(d: Dispatch) -> anyhow::Result<()> {
    static ONCE: Once = Once::new();
    static MAIN_DISPATCH: Mutex<Option<(LevelFilter, Box<dyn Log>)>> = Mutex::new(None);
    ONCE.call_once(|| {
        Dispatch::new()
            .chain(Output::call(|record| {
                if let Some((level, log)) = &*MAIN_DISPATCH.lock().unwrap() {
                    if record.level() <= *level {
                        log.log(record);
                    }
                }
            }))
            .apply()
            .expect("Error setting main log dispatch");
    });

    *MAIN_DISPATCH.lock().unwrap() = Some(d.into_log());
    Ok(())
}

trait MsgFormatter: Send + Sync {
    fn format(&self, out: FormatCallback, msg: &Arguments, record: &Record);
}

struct ColorMsgFormatter {
    line_colors: ColoredLevelConfig,
    level_colors: ColoredLevelConfig,
}

#[derive(Clone, Copy)]
struct NoColorMsgFormatter;

impl ColorMsgFormatter {
    pub fn new() -> Self {
        let line_colors = ColoredLevelConfig::new();
        Self {
            level_colors: line_colors.info(Color::Green),
            line_colors,
        }
    }
}

impl MsgFormatter for ColorMsgFormatter {
    fn format(&self, out: FormatCallback, msg: &Arguments, record: &Record) {
        out.finish(format_args!(
            "{}[{}] [{}] {}\x1B[0m",
            format_args!(
                "\x1B[{}m",
                self.line_colors.get_color(&record.level()).to_fg_str()
            ),
            self.level_colors.color(record.level()),
            record.target(),
            msg
        ));
    }
}

impl NoColorMsgFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl MsgFormatter for NoColorMsgFormatter {
    fn format(&self, out: FormatCallback, msg: &Arguments, record: &Record) {
        out.finish(format_args!(
            "[{}] [{}] {}",
            record.level(),
            record.target(),
            msg
        ));
    }
}

pub fn init_log() -> anyhow::Result<()> {
    let no_color = std::env::args().any(|arg| arg == "--no-color");
    let no_color_formatter = NoColorMsgFormatter::new();
    let stdout_formatter: Box<dyn MsgFormatter> = match no_color {
        true => Box::new(no_color_formatter),
        false => Box::new(ColorMsgFormatter::new()),
    };

    set_dispatch(
        Dispatch::new()
            .level_for("amk", LevelFilter::Trace)
            .level(LevelFilter::Info)
            .chain(
                Dispatch::new()
                    .format(move |out, msg, record| no_color_formatter.format(out, msg, record))
                    .chain(fern::log_file("amk.log")?),
            )
            .chain(
                Dispatch::new()
                    .format(move |out, msg, record| stdout_formatter.format(out, msg, record))
                    .chain(std::io::stderr()),
            ),
    )?;
    log::trace!("Logger initialized");
    log::debug!("Logger initialized");
    log::info!("Logger initialized");
    log::warn!("Logger initialized");
    log::error!("Logger initialized");
    Ok(())
}
