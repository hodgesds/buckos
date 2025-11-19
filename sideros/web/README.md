# Sideros Web

The official website and documentation server for Sideros.

## Features

- Home page with project overview
- Installation guide with step-by-step instructions
- Wiki with comprehensive documentation
- About page with project information

## Running

```bash
cd sideros/web
cargo run
```

The server will start at `http://localhost:3000`.

## Pages

- `/` - Home page
- `/install` - Installation instructions
- `/wiki` - Documentation wiki
- `/about` - About the project

## Development

The website uses:
- **Axum** - Web framework
- **Askama** - Type-safe HTML templating
- **Tower-HTTP** - Static file serving

Templates are located in `templates/` and static assets in `static/`.

## Building

```bash
cargo build --release
```

The release binary will be at `target/release/sideros-web`.
