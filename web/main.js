/*jshint browser:true,curly: false */
/* global L */

window.onload = () => {
    window.vue = new Vue({
        el: '#app',
        data: {
            credentials: {
                map: { key: 'pk.eyJ1Ijoic2JtYTQ0IiwiYSI6ImNpcXNycTNqaTAwMDdmcG5seDBoYjVkZGcifQ.ZVIe6sjh0QGeMsHpBvlsEA' }
            },
            delta: false,
            deltas: [],
        },
        created: function() {
            this.deltas_refresh();
        },
        watch: { },
        methods: {
            deltas_refresh: function() {
                fetch('http://localhost:8000/api/deltas').then((response) => {
                      return response.json();
                }).then((body) => {
                    this.deltas.splice(0, this.deltas.length);
                    this.deltas = this.deltas.concat(body);
                });
            },
            delta_get: function(delta_id) {
                if (!delta_id) return;

                fetch(`http://localhost:8000/api/delta/${delta_id}`).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.delta = body;
                });
            }
        }
    });

    window.vue.moment = moment;

    mapboxgl.accessToken = window.vue.credentials.map.key;
    window.vue.map = new mapboxgl.Map({
        container: 'map',
        style: 'mapbox://styles/mapbox/streets-v8',
        center: [-96, 37.8],
        zoom: 3
    });

    window.vue.map.on('load', () => {
        window.vue.map.addLayer({
            id: 'hecate-data',
            type: 'circle',
            source: {
                type: 'vector',
                tiles: ['http://localhost:8000/api/tiles/{z}/{x}/{y}']
            },
            'source-layer': 'data',
            paint: {
                'circle-radius': {
                    base: 1.75,
                    stops: [[14, 2], [22, 25]]
                },
            }
        });
    });
}
