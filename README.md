<h1 align='center'>Hecate</h1>

## Brief

OpenStreetMap Inspired Data Storage Backend Focused on Speed and GeoJSON Interchange

## Docker File (Coverage Tests)

The Docker file is designed to give the user a testing environment to get tests up and running and be able to view coverage information.

Install docker and then run

```
docker build .

docker run  --security-opt seccomp=unconfined {{HASH FROM ABOVE}}
```

The --security-opt flag is required to be able to run and view `kcov` output.

## API

### Index

#### `GET` `/`

### User Options

#### `GET` `/api/user/create`

Create a new user, provied the username & password are not already taken

*Required Options*

| Option     | Notes |
| ---------- | ----- |
| `username` | Desired username, must be unique |
| `password` | Desired password |
| `email`    | Desired email, must be unique |

*Example*

```bash
curl +X GET 'http://localhost:8000/api/user/create?ingalls&password=yeaheh&email=ingalls@protonmail.com
```

---

### Boundary Downloading

#### `GET` `/api/data/bounds/`

Return an array of possible boundary files with which data can be extracted from the server with

*Example*

```bash
curl +X GET 'http://localhost:8000/api/data/bounds
```

---

#### `GET` `/api/data/bounds/<bounds>`

Return a `FeatureCollection` of all the geometries within the specified boundary file.

*Required Options*

| Option     | Notes |
| ---------- | ----- |
| `<bounds>` | One of the boundary files as specified via the `/ap/data/bounds`

*Example*

```bash
curl +X GET 'http://localhost:8000/api/data/bounds/us_dc
```

---
