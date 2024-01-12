use std::str::FromStr;

pub fn init_logger(module_names: Vec<String>, log_level: &str) {
    use chrono::Utc;
    use env_logger::{fmt::Color, Builder};
    use log::LevelFilter;
    use std::io::Write;

    let level: LevelFilter = LevelFilter::from_str(log_level).unwrap();
    let mut builder: Builder = Builder::new();
    for module_name in module_names {
        builder.filter(Some(module_name.as_str()), level);
    }
    builder
        .filter(None, LevelFilter::Off)
        .format(|buf, record| {
            let mut style = buf.style();
            match record.level() {
                log::Level::Error => style.set_color(Color::Red),
                log::Level::Warn => style.set_color(Color::Yellow),
                log::Level::Debug => style.set_color(Color::Blue),
                _ => &mut style,
            };
            style.set_bold(true);
            let now = Utc::now();
            let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();
            writeln!(buf, "[{}]: {}", ts, style.value(record.args()),)
        })
        .init();
}
