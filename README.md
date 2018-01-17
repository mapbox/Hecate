<h1 align='center'>Hecate</h1>

## Brief

OpenStreetMap Inspired Data Storage Backend Focused on Performance and GeoJSON Interchange

## Table Of Contents

1. [Brief](#brief)
2. [Table of Contents](#table-of-contents)
3. [Docker File](#docker-file-coverage-tests)
4. [Feature Format](#feature-format)
5. [API](#api)
    - [User Options](#user-options)
    - [Downloading via Boundaries](#downloading-via-boundaries)
    - [Downloading Individual Features](#downloading-individual-features)
    - [Downloading Multiple Features via BBOX](#downloading-multiple-features-via-bbox)
    - [Feature Creation](#feature-creation)

## Docker File (Coverage Tests)

The Docker file is designed to give the user a testing environment to get tests up and running and be able to view coverage information.

Install docker and then run

```
docker build .

docker run  --security-opt seccomp=unconfined {{HASH FROM ABOVE}}
```

The --security-opt flag is required to be able to run and view `kcov` output.

## Feature Format

Hecate is designed as a GeoJSON first interchange and uses [standard GeoJSON](http://geojson.org/) with a couple additions
and exceptions as outlined below.

*Supported Geometry Types*
- `Point`
- `MultiPoint`
- `LineString`
- `MultiLineString`
- `Polygon`
- `MultiPolygon`

*Unsupported Geometry Types*
- `GeometryCollection`

### Additional Memebers

The following table outlines top-level members used by hecate to handle feature creation/modification/deletion.

Key/Value pairs in the `.properties` of a given feature are _never_ directly used by the server and are simply
passed through to the storage backend. This prevents potential conflicts between user properties and required
server members.

| Member    | Notes |
| :-------: | ----- |
| `id`      | The unique integer `id` of a given feature. Note that all features get a unique id accross GeoJSON Geometry Type |
| `version` | The version of a given feature, starts at `1` for a newly created feature |
| `action`  | Only used for uploads, the desired action to be performed. One of `create`, `modify` or `delete` |


### Samples

## API

### Index

#### `GET` `/`

Healthcheck URL, currently returns `Hello World!`

*Example*

```bash
curl +X GET 'http://localhost:8000/
```

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

<h3 align='center'>Feature Creation</h3>

#### `POST` `/api/data/feature`

Create, Modify, or Delete an individual GeoJSON `Feature`

---

#### `POST` `/api/data/features`

Create, Modify, and/or Delete many features via a GeoJSON `FeatureCollection`

---

