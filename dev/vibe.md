```shell
cd c:\Users\hrag\Sync\Programming\rust\passworder
aider --model openai/gemma-4-31b --chat-mode ask --no-verify-ssl
```

/add Cargo.toml src\main.rs


This is a rust project with iced v0.14 gui. It uses tray_icon to show an icon in the system bar.

**vibe**
This is a rust project with iced v0.14 gui.
I get this error when clicking "Generate" in the tray icon menu:
```
thread '<unnamed>' (20340) panicked at src\main.rs:134:21:
there is no reactor running, must be called from the context of a Tokio 1.x runtime
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

/code Make the changes. You've already planned above, so no need to replan.
