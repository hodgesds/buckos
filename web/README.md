# buckos-web

The official website and documentation server for Buckos.

## Overview

`buckos-web` provides the official Buckos website, including project documentation, installation guides, and wiki content. It's built with Axum and Askama for fast, type-safe web serving.

## Features

- **Project Homepage**: Overview and introduction to Buckos
- **Installation Guide**: Step-by-step installation instructions
- **Documentation Wiki**: Comprehensive system documentation
- **About Page**: Project information and acknowledgments
- **Static Asset Serving**: CSS, JavaScript, and images
- **Type-Safe Templates**: Compile-time checked HTML templates

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
buckos-web = { path = "../web" }
```

Or install the binary:

```bash
cargo install --path web
```

## Running

### Development

```bash
cd web
cargo run
```

The server will start at `http://localhost:3000`.

### Production

```bash
cargo build --release
./target/release/buckos-web
```

### With Custom Port

```bash
BUCKOS_WEB_PORT=8080 buckos-web
```

## Pages

| Route | Description |
|-------|-------------|
| `/` | Home page with project overview |
| `/install` | Installation instructions |
| `/wiki` | Documentation wiki |
| `/about` | About the project |

## Project Structure

```
buckos-web/
├── src/
│   └── main.rs           # Server and routing
├── templates/
│   ├── base.html         # Base layout
│   ├── index.html        # Home page
│   ├── install.html      # Installation guide
│   ├── wiki.html         # Wiki/documentation
│   └── about.html        # About page
└── static/
    ├── css/              # Stylesheets
    │   └── style.css
    ├── js/               # JavaScript
    │   └── main.js
    └── images/           # Images and assets
        └── logo.png
```

## Development

### Technology Stack

- **[Axum](https://github.com/tokio-rs/axum)** - Web framework
- **[Askama](https://github.com/djc/askama)** - Type-safe templating
- **[Tower-HTTP](https://github.com/tower-rs/tower-http)** - HTTP utilities
- **[Tokio](https://tokio.rs/)** - Async runtime

### Adding a New Page

1. Create a template in `templates/`:

```html
{% extends "base.html" %}

{% block title %}Page Title{% endblock %}

{% block content %}
<div class="container">
    <h1>Page Title</h1>
    <p>Page content...</p>
</div>
{% endblock %}
```

2. Add the route in `src/main.rs`:

```rust
#[derive(Template)]
#[template(path = "newpage.html")]
struct NewPageTemplate {
    // Template variables
}

async fn new_page() -> impl IntoResponse {
    NewPageTemplate { }
}

// In router setup:
.route("/newpage", get(new_page))
```

### Templates

Templates use Askama syntax (similar to Jinja2):

```html
{% extends "base.html" %}

{% block content %}
<h1>{{ title }}</h1>

{% for item in items %}
    <li>{{ item.name }}</li>
{% endfor %}

{% if show_footer %}
    <footer>...</footer>
{% endif %}
{% endblock %}
```

### Static Files

Static files are served from the `static/` directory:

```html
<link rel="stylesheet" href="/static/css/style.css">
<script src="/static/js/main.js"></script>
<img src="/static/images/logo.png" alt="Logo">
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BUCKOS_WEB_PORT` | `3000` | Server port |
| `BUCKOS_WEB_HOST` | `0.0.0.0` | Bind address |
| `RUST_LOG` | `info` | Log level |

### Configuration File

```toml
# /etc/web.toml

[server]
host = "0.0.0.0"
port = 3000

[logging]
level = "info"

[content]
wiki_path = "/var/lib/buckos/wiki"
```

## API Endpoints

While primarily a website, `buckos-web` also provides some API endpoints:

```bash
# Get system status (planned)
GET /api/status

# Get package information (planned)
GET /api/packages/{name}

# Search documentation (planned)
GET /api/search?q=query
```

## Building

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

The release binary will be at `target/release/buckos-web`.

### Docker (Planned)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/buckos-web /usr/local/bin/
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static
CMD ["buckos-web"]
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.7 | Web framework |
| `tokio` | 1.0 | Async runtime |
| `askama` | 0.12 | Templating |
| `askama_axum` | 0.4 | Axum integration |
| `tower-http` | 0.5 | Static file serving |

## Content Guidelines

### Documentation Style

- Use clear, concise language
- Include examples and code snippets
- Link to related topics
- Keep content up to date

### Code Examples

```html
<pre><code class="language-bash">
# Install a package
buckos install www-client/firefox
</code></pre>
```

### Screenshots

- Use PNG format
- Include alt text
- Keep file sizes reasonable
- Place in `static/images/`

## Testing

```bash
# Run tests
cargo test -p buckos-web

# Test with coverage
cargo tarpaulin -p buckos-web

# Integration tests
cargo test -p buckos-web --features integration
```

## Performance

### Optimization Tips

- Templates are compiled at build time
- Static files can be served via CDN
- Enable gzip compression in production
- Use caching headers appropriately

### Benchmark

```bash
# Using wrk
wrk -t12 -c400 -d30s http://localhost:3000/
```

## Contributing

When contributing to the website:

1. Follow the existing template structure
2. Ensure pages are responsive
3. Test across browsers
4. Optimize images
5. Keep accessibility in mind

### Running Locally

```bash
# Clone and build
git clone https://github.com/hodgesds/buckos.git
cd buckos/web
cargo run

# Open browser
open http://localhost:3000
```

## License

This crate is part of the Buckos project and is licensed under the same terms.

## See Also

- [Axum Documentation](https://docs.rs/axum)
- [Askama Documentation](https://djc.github.io/askama/)
- [Buckos Project](https://github.com/hodgesds/buckos)
