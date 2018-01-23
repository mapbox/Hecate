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
                if (!this.delta) return;
               
                this.delta.bbox = turf.bbox(this.delta.features);
                this.map.fitBounds(this.delta.bbox);
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
                    this.delta = body;
                });
            },
            map_style: function() {
                const foregroundColor = '#FF0000';

                this.map.addSource('hecate-data', {
                    type: 'vector',
                    tiles: [ 'http://127.0.0.1:8000/api/tiles/{z}/{x}/{y}' ]
                });

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
        window.vue.map_style();
    });

    window.vue.map.addControl(new MapboxGeocoder({
        accessToken: mapboxgl.accessToken,
    }));
}

