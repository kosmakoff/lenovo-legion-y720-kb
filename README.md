This repository contains the userspace utilities to control the backlighting of Lenovo Legion Y720-i15kb laptop.

### Building the code

```bash
cargo build --release
```

### lenovo-kb-light

Reads persisted settings and turns backlighting `ON` or `OFF`.

### lenovo-kb-lightd

Userspace daemon that listens for Fn+Space key combination and toggles the backlighting. New state is persisted in settings file.

