# geologs

Access logs parser that creates a small html report with analytics.

- Uses [ipgeolocation.io](https://ipgeolocation.io) as a source of geodata.
- Uses [Tabler](https://tabler.io) for HTML report.

## Install

Download binary from [releases page](https://github.com/tetafro/geologs/releases).

## Run

Signup on [ipgeolocation.io](https://ipgeolocation.io) to get a free API key.

```sh
geologs --api-key qwerty --file ./access.log
```

## Development

Install dependencies
```sh
make deps
```

Build
```sh
make debug # debug version
make build # release version
```

Run
```sh
make run
```

Update Javascript and CSS dependencies
```sh
tabler=1.0.0-beta17
curl -L -o static/tabler.js \
    "https://cdn.jsdelivr.net/npm/@tabler/core@${tabler}/dist/js/tabler.min.js"
curl -L -o static/tabler.min.css \
    "https://cdn.jsdelivr.net/npm/@tabler/core@${tabler}/dist/css/tabler.min.css"
curl -L -o static/jsvectormap.js \
    https://cdn.jsdelivr.net/npm/jsvectormap
curl -L -o static/world.js \
    https://cdn.jsdelivr.net/npm/jsvectormap/dist/maps/world.js
curl -L -o static/jsvectormap.min.css \
    https://cdn.jsdelivr.net/npm/jsvectormap/dist/css/jsvectormap.min.css
```
