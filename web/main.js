1.25/*jshint browser:true,curly: false */
/* global L */

window.onload = () => {
    window.vue = new Vue({
        el: '#app',
        data: {
            credentials: {
                map: { key: 'pk.eyJ1Ijoic2JtYTQ0IiwiYSI6ImNpcXNycTNqaTAwMDdmcG5seDBoYjVkZGcifQ.ZVIe6sjh0QGeMsHpBvlsEA' }
            },
            delta: false,
            deltas: []
        },
        created: function() {
            this.deltas_refresh();
        },
        watch: {
            delta: function() {
                //Reset Normal Map
                if (!this.delta) {
                    this.map_delta_unstyle();
                    this.map_data_style();

                    this.map.getSource('hecate-delta').setData({ type: 'FeatureCollection', features: [] });
                } else {
                    this.map.getSource('hecate-delta').setData(this.delta.features);

                    this.map_default_unstyle();
                    this.map_delta_style();

                    this.delta.bbox = turf.bbox(this.delta.features);
                    this.map.fitBounds(this.delta.bbox);
                }
            }
        },
        methods: {
            deltas_refresh: function() {
                fetch('http://127.0.0.1:8000/api/deltas').then((response) => {
                      return response.json();
                }).then((body) => {
                    this.deltas.splice(0, this.deltas.length);
                    this.deltas = this.deltas.concat(body);
                });
            },
            delta_get: function(delta_id) {
                if (!delta_id) return;

                fetch(`http://127.0.0.1:8000/api/delta/${delta_id}`).then((response) => {
                      return response.json();
                }).then((body) => {
                    body.features.features = body.features.features.map(feat => {
                        feat.properties._action = feat.action;
                        return feat;
                    });
                    this.delta = body;
                });
            },
            map_delta_unstyle: function() {
                this.map.removeLayer('hecate-delta-polygons');
                this.map.removeLayer('hecate-delta-polygon-outlines');
                this.map.removeLayer('hecate-delta-lines');
                this.map.removeLayer('hecate-delta-points');
            },
            map_delta_style: function() {
                let action_create = '#008000';
                let action_modify = '#FFFF00';
                let action_delete = '#FF0000';
                
                this.map.addLayer({
                    id: 'hecate-data-polygons',
                    type: 'fill',
                    source: 'hecate-delta',
                    filter: ['==', '$type', 'Polygon'],
                    paint: {
                        'fill-opacity': 0.4,
                        'fill-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ]
                    }
                });

                this.map.addLayer({
                    id: 'hecate-data-polygon-outlines',
                    type: 'line',
                    source: 'hecate-delta',
                    filter: ['==', '$type', 'Polygon'],
                    layout: {
                        'line-join': 'round',
                        'line-cap': 'round'
                    },
                    paint: {
                        'line-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ],
                        'line-width': 0.75
                    }
                })

                this.map.addLayer({
                    id: 'hecate-data-lines',
                    type: 'line',
                    source: 'hecate-delta',
                    filter: ['==', '$type', 'LineString'],
                    layout: {
                        'line-join': 'round',
                        'line-cap': 'round'
                    },
                    paint: {
                        'line-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ],
                        'line-width': 1.25
                    }
                });

                this.map.addLayer({
                    id: 'hecate-data-points',
                    type: 'circle',
                    source: 'hecate-delta',
                    filter: ['==', '$type', 'Point'],
                    paint: {
                        'circle-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ],
                        'circle-radius': 4
                    }
                });

            },
            map_default_unstyle: function() {
                this.map.removeLayer('hecate-data-polygons');
                this.map.removeLayer('hecate-data-polygon-outlines');
                this.map.removeLayer('hecate-data-lines');
                this.map.removeLayer('hecate-data-points');
            },
            map_default_style: function() {
                const foregroundColor = '#FF0000';
                this.map.addLayer({
                    id: 'hecate-data-polygons',
                    type: 'fill',
                    source: 'hecate-data',
                    'source-layer': 'data',
                    filter: ['==', '$type', 'Polygon'],
                    paint: {
                        'fill-opacity': 0.1,
                        'fill-color': foregroundColor
                    }
                });

                this.map.addLayer({
                    id: 'hecate-data-polygon-outlines',
                    type: 'line',
                    source: 'hecate-data',
                    'source-layer': 'data',
                    filter: ['==', '$type', 'Polygon'],
                    layout: {
                        'line-join': 'round',
                        'line-cap': 'round'
                    },
                    paint: {
                        'line-color': foregroundColor,
                        'line-width': 0.75,
                        'line-opacity': 0.75
                    }
                })

                this.map.addLayer({
                    id: 'hecate-data-lines',
                    type: 'line',
                    source: 'hecate-data',
                    'source-layer': 'data',
                    filter: ['==', '$type', 'LineString'],
                    layout: {
                        'line-join': 'round',
                        'line-cap': 'round'
                    },
                    paint: {
                        'line-color': foregroundColor,
                        'line-width': 1.25,
                        'line-opacity': 0.75
                    }
                });

                this.map.addLayer({
                    id: 'hecate-data-points',
                    type: 'circle',
                    source: 'hecate-data',
                    'source-layer': 'data',
                    filter: ['==', '$type', 'Point'],
                    paint: {
                        'circle-color': foregroundColor,
                        'circle-radius': 4,
                        'circle-opacity': 0.85
                    }
                });

            }
        }
    });

    window.vue.moment = moment;

    mapboxgl.accessToken = window.vue.credentials.map.key;
    window.vue.map = new mapboxgl.Map({
        container: 'map',
        style: 'mapbox://styles/mapbox/satellite-v9',
        center: [-96, 37.8],
        zoom: 3
    });

    window.vue.map.on('load', () => {
        window.vue.map.addSource('hecate-data', {
            type: 'vector',
            tiles: [ 'http://127.0.0.1:8000/api/tiles/{z}/{x}/{y}' ]
        });

        window.vue.map.addSource('hecate-delta', {
            type: 'geojson',
            data: { type: 'FeatureCollection', features: [] }
        });

        window.vue.map_default_style();
    });

    window.vue.map.addControl(new MapboxGeocoder({
        accessToken: mapboxgl.accessToken,
    }));
}

