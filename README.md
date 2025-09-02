A minimal and pretty logger for the log crate.

```rs
use log::LevelFilter;

pretty_logging::init(LevelFilter::Trace, ["my_crate", "axum"]);
```
