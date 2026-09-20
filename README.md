# Age2Scenario

Age2Scenario is a free and open-source archive and sharing platform for custom scenarios for **Age of Empires II: Definitive Edition**.

The goal of the project is to provide a modern place to preserve, discover, and share community-created scenarios, including older scenarios that might otherwise become difficult to find.

> Age2Scenario is an unofficial community project and is not affiliated with or endorsed by Microsoft.

## Status

Age2Scenario is currently in early development.

The initial implementation is focused on establishing the application and deployment infrastructure before implementing scenario storage, search, accounts, metadata, and other features.

Current application endpoints:

- `/` — web frontend
- `/api/hello` — example Rust API endpoint
- `/healthz` — application health check

## Technology

The current application uses:

- Rust
- Axum
- Tokio
- HTML / JavaScript
- Docker
- GitHub Actions

The frontend and API are currently served by the same Rust application.

## Development

Run the application locally:

```bash
cargo run
```

Then open:

```text
http://localhost:8080
```

The health endpoint can be checked with:

```bash
curl http://localhost:8080/healthz
```

### Docker

Build the development image:

```bash
docker build -t age2scenario:dev .
```

Run it:

```bash
docker run --rm -p 8080:8080 age2scenario:dev
```

Then open:

```text
http://localhost:8080
```

## License

Age2Scenario is free software licensed under the **GNU Affero General Public License, version 3 or (at your option) any later version**.

Copyright (C) 2026 Julià Mauri Costa

See [LICENSE](LICENSE) for the full license text.

The AGPL applies to the **Age2Scenario software and source code**.

Scenario files, user uploads, and other content stored or distributed by an Age2Scenario instance are **not licensed under the AGPL merely by being hosted by the software**. Such content remains subject to its own copyright and licensing terms.
