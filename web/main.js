/*jshint browser:true,curly: false */
/* global L */

window.onload = () => {
    window.vue = new Vue({
        el: '#app',
        data: {
            credentials: {
                map: { key: 'pk.eyJ1Ijoic2JtYTQ0IiwiYSI6ImNpcXNycTNqaTAwMDdmcG5seDBoYjVkZGcifQ.ZVIe6sjh0QGeMsHpBvlsEA' }
            },
            deltas: []
        },
        created: function() {
            fetch('http://localhost:8000/api/deltas').then((response) => {
                  return response.json();
            }).then((body) => {
                this.deltas = this.deltas.concat(body);
            });
        },
        watch: { },
        methods: { }
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
