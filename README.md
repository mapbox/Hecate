<h1 align='center'>Hecate</h1>

## Brief

OpenStreetMap Inspired Data Storage Backend Focused on Speed and GeoJSON Interchange

## Table Of Contents

1. [Brief](#brief)
2. [Table of Contents](#table-of-content)
3. [Docker File](##docker-file-coverage-tests)
4. [API](##api)

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

---

<h3 align='center'>User Options</h3>

#### `GET` `/api/user/create`

Create a new user, provied the username & password are not already taken

*Required Options*

| Option     | Notes |
| :--------: | ----- |
| `username` | `REQUIRED` Desired username, must be unique |
| `password` | `REQUIRED` Desired password |
| `email`    | `REQUIRED` Desired email, must be unique |

*Example*

```bash
curl +X GET 'http://localhost:8000/api/user/create?ingalls&password=yeaheh&email=ingalls@protonmail.com
```

---

<h3 align='center'>Downloading via Boundaries</h3>

#### `GET` `/api/data/bounds/`

Return an array of possible boundary files with which data can be extracted from the server with

*Example*

```bash
curl +X GET 'http://localhost:8000/api/data/bounds
```

---

#### `GET` `/api/data/bounds/<bounds>`

Return line delimited GeoJSON `Feature` of all the geometries within the specified boundary file.

*Required Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | One of the boundary files as specified via the `/ap/data/bounds` |

*Example*

```bash
curl +X GET 'http://localhost:8000/api/data/bounds/us_dc
```

---

<h3 align='center'>Downloading Individual Features</h3>

#### `GET` `/api/data/feature/<id>`

Return a single GeoJSON `Feature` given its' ID.

*Required Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | Numeric ID of a given feature to download |

*Example*

```bash
curl +X GET 'http://localhost:8000/api/data/features/1542
```

---

<h3 align='center'>Downloading Multiple Features via BBOX</h3>

#### `GET` `/api/data/features`

Return a `FeatureCollection` of all features within a given bbox

*Required Options*

| Option | Notes |
| :----: | ----- |
| `bbox` | `REQUIRED` Bounding Box in format `left,bottom,right,top` |

 ---
