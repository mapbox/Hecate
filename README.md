# Hecate
OpenStreetMap Inspired Data Storage Backend Focused on Speed and GeoJSON Interchange

## Docker File

The Docker file is designed to give the user a testing environment to get tests up and running and be able to view coverage information.

Install docker and then run

```
docker build .

docker run  --security-opt seccomp=unconfined {{HASH FROM ABOVE}}
```

The --security-opt flag is required to be able to run and view `kcov` output.
