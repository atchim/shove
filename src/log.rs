use atty::{Stream, is as is_tty};
use lazy_static::lazy_static;
use log::{
  Level,
  LevelFilter,
  Log,
  Metadata,
  Record,
  max_level,
  set_logger,
  set_max_level,
};
use std::{io::Write, panic::set_hook, process::exit, sync::Once};
use super::cli::ColorWhen;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

lazy_static! {
  static ref BOLD_GREEN_SPEC: ColorSpec = ColorSpec::new()
    .set_bold(true)
    .set_fg(Some(Color::Ansi256(2)))
    .to_owned();

  static ref BOLD_MAGENTA_SPEC: ColorSpec = ColorSpec::new()
    .set_bold(true)
    .set_fg(Some(Color::Ansi256(13)))
    .to_owned();

  static ref BOLD_ORANGE_SPEC: ColorSpec = ColorSpec::new()
    .set_bold(true)
    .set_fg(Some(Color::Ansi256(3)))
    .to_owned();

  static ref BOLD_RED_SPEC: ColorSpec = ColorSpec::new()
    .set_bold(true)
    .set_fg(Some(Color::Ansi256(1)))
    .to_owned();

  static ref BOLD_YELLOW_SPEC: ColorSpec = ColorSpec::new()
    .set_bold(true)
    .set_fg(Some(Color::Ansi256(11)))
    .to_owned();

  static ref DEFAULT_SPEC: ColorSpec = ColorSpec::default();
}

static INIT: Once = Once::new();

static mut LOGGER: Logger = Logger {
  berserker: false,
  stderr_choice: ColorChoice::Auto,
  stdout_choice: ColorChoice::Auto,
};

struct Logger {
  berserker: bool,
  stderr_choice: ColorChoice,
  stdout_choice: ColorChoice,
}

impl Log for Logger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= max_level()
  }

  fn flush(&self) {}

  fn log(&self, record: &Record) {
    if !self.enabled(record.metadata()) {
      return;
    }

    if let Level::Error = record.level() {
      let mut stderr = StandardStream::stderr(self.stderr_choice);
      stderr.set_color(&BOLD_RED_SPEC).unwrap();
      write!(stderr, "ERROR").unwrap();
      stderr.set_color(&DEFAULT_SPEC).unwrap();
      writeln!(stderr, ": {}", record.args()).unwrap();

      match self.berserker {
        false => exit(1),
        true => return,
      }
    }

    let mut stdout = StandardStream::stdout(self.stdout_choice);

    match record.level() {
      Level::Error => unreachable!(),
      Level::Warn => {
        stdout.set_color(&BOLD_ORANGE_SPEC).unwrap();
        write!(stdout, "WARN").unwrap();
      }
      Level::Info => {
        stdout.set_color(&BOLD_GREEN_SPEC).unwrap();
        write!(stdout, "INFO").unwrap();
      }
      Level::Debug => {
        stdout.set_color(&BOLD_YELLOW_SPEC).unwrap();
        write!(stdout, "DEBUG").unwrap();
      }
      Level::Trace => {
        stdout.set_color(&BOLD_MAGENTA_SPEC).unwrap();
        write!(stdout, "TRACE").unwrap();
      }
    }

    stdout.set_color(&DEFAULT_SPEC).unwrap();
    writeln!(stdout, ": {}", record.args()).unwrap();
  }
}

pub fn init(verbose: usize, berserker: bool, color: ColorWhen) {
  INIT.call_once(|| {
    let stderr_choice: ColorChoice;
    let stdout_choice: ColorChoice;

    match color {
      ColorWhen::Always => {
        stderr_choice = ColorChoice::AlwaysAnsi;
        stdout_choice = stderr_choice;
      }
      ColorWhen::Auto => {
        match is_tty(Stream::Stderr) {
          false => stderr_choice = ColorChoice::Never,
          true => stderr_choice = ColorChoice::Auto,
        }
        match is_tty(Stream::Stdout) {
          false => stdout_choice = ColorChoice::Never,
          true => stdout_choice = ColorChoice::Auto,
        }
      }
      ColorWhen::Never => {
        stderr_choice = ColorChoice::Never;
        stdout_choice = stderr_choice;
      }
    }

    unsafe {
      LOGGER.berserker = berserker;
      LOGGER.stderr_choice = stderr_choice;
      LOGGER.stdout_choice = stdout_choice;
      set_logger(&LOGGER).unwrap();
    }

    set_max_level(match verbose {
      0 => LevelFilter::Error,
      1 => LevelFilter::Warn,
      2 => LevelFilter::Info,
      3 => LevelFilter::Debug,
      _ => LevelFilter::Trace,
    });

    // Make the panic output more familiar with the log output.
    set_hook(Box::new(|info| {
      let mut stderr = StandardStream::stderr(unsafe {LOGGER.stderr_choice});
      stderr.set_color(&BOLD_RED_SPEC).unwrap();
      write!(stderr, "PANIC").unwrap();
      stderr.set_color(&DEFAULT_SPEC).unwrap();
      writeln!(stderr, ": {}", &info).unwrap();
    }));
  });
}
