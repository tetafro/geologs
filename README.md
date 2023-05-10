# geologs

Access logs parser that creates a small html report with analytics.

## Config

Copy config and populate values
```sh
cp .env.example .env
```

## Run

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

## Dependencies

- [Tabler](https://tabler.io/docs/getting-started/download)
- [JSVectorMap](https://jvm-docs.vercel.app/docs/installation#cdn)

Update
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
