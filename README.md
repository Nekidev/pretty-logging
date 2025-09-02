A minimal and pretty logger for the log crate.

```rs
use log::LevelFilter;

pretty_logging::init(LevelFilter::Trace, []);

log::trace!("Hello world!");
log::debug!("Hello world!");
log::info!("Hello world!");
log::warn!("Hello world!");
log::error!("Hello world!");
panic!("Hello world!");
```

You'll see an output as follows:

```
02/09/2025 at 01:02:21.61 [TRACE] Hello world!
02/09/2025 at 01:02:21.61 [DEBUG] Hello world!
02/09/2025 at 01:02:21.61 [INFO]  Hello world!
02/09/2025 at 01:02:21.61 [WARN]  Hello world!
02/09/2025 at 01:02:21.61 [ERROR] Hello world!
02/09/2025 at 01:02:21.61 [PANIC] Hello world!
```

The real output has colors. Check it out!
