use structopt::clap::AppSettings;
use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, IntoStaticStr};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use std::env;

fn main() {
    let opt = Opt::from_args();

    FmtSubscriber::builder()
        .without_time()
        .with_ansi(opt.color.should_enable_ansi_for_stderr())
        .with_max_level(Level::INFO)
        .init();

    if let Err(err) = opt.run() {
        for (i, err) in err.chain().enumerate() {
            for (j, line) in err.to_string().lines().enumerate() {
                match (i, j) {
                    (0, _) => error!("{}", line),
                    (_, 0) => error!("Caused by: {}", line),
                    (_, _) => error!("           {}", line),
                }
            }
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(author, about, setting(AppSettings::DeriveDisplayOrder))]
struct Opt {
    #[structopt(
        long,
        value_name("WHEN"),
        default_value(ColorChoice::default().into()),
        possible_values(ColorChoice::variants()),
        help("Coloring")
    )]
    color: ColorChoice,
}

impl Opt {
    fn run(self) -> anyhow::Result<()> {
        info!("Hello, world!");
        Ok(())
    }
}

#[derive(Debug, EnumString, IntoStaticStr, EnumVariantNames, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
enum ColorChoice {
    Auto,
    Never,
    Always,
}

impl ColorChoice {
    fn should_enable_ansi_for_stderr(self) -> bool {
        #[cfg(not(windows))]
        fn on_auto() -> bool {
            atty::is(atty::Stream::Stderr) && env::var("TERM").ok().map_or(false, |v| v != "dumb")
        }

        #[cfg(windows)]
        fn on_auto() -> bool {
            use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            use winapi_util::HandleRef;

            use std::ops::Deref;

            let term = env::var("TERM");
            let term = term.as_ref().map(Deref::deref);
            if term == Ok("dumb") || term == Ok("cygwin") {
                false
            } else if env::var_os("MSYSTEM").is_some() && term.is_ok() {
                atty::is(atty::Stream::Stderr)
            } else {
                atty::is(atty::Stream::Stderr)
                    && winapi_util::console::mode(HandleRef::stderr())
                        .ok()
                        .map_or(false, |m| m & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0)
            }
        }

        match self {
            Self::Auto => on_auto(),
            Self::Never => false,
            Self::Always => true,
        }
    }
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}
