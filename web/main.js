/*jshint browser:true,curly: false */
/* global L */

window.onload = () => {
    window.vue = new Vue({
        el: '#app',
        data: {
            credentials: {
                map: { key: 'pk.eyJ1Ijoic2JtYTQ0IiwiYSI6ImNpcXNycTNqaTAwMDdmcG5seDBoYjVkZGcifQ.ZVIe6sjh0QGeMsHpBvlsEA' },
                authed: false
            },
            panel: { panel: 'deltas' },
            modal: {
                type: false,
                ok: {
                    header: '',
                    body: ''
                },
                login: {
                    username: '',
                    password: ''
                },
                register: {
                    username: '',
                    password: '',
                    email: ''
                }
            },
            feature: false,
            delta: false,
            deltas: [],
            bounds: false,
        },
        components: {
            heading: {
                template: `
                    <div class='flex-child px12 py12'>
                        <h3 class='fl py6 txt-m txt-bold' v-text='title'></h3>
                        <button v-if='is_authed' @click='logout' class='fr px12 py12 btn round bg-white bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use xlink:href='#icon-logout'></use></svg></button>
                        <button @click='login_show' class='fr px12 py12 btn round bg-white bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use xlink:href='#icon-user'></use></svg></button>
                    </div>
                `,
                props: [ 'is_authed', 'login_show', 'logout', 'title' ],
            }
        },
        created: function() {
            this.deltas_refresh();
        },
        watch: {
            panel: function() {
                if (this.panel.panel === 'deltas') {
                    this.bounds = false;
                } else {
                    this.bounds_refresh();
                }
            },
            delta: function() {
                //Reset Normal Map
                if (!this.delta) {
                    this.map_default_style();

                    this.map.getSource('hecate-delta').setData({ type: 'FeatureCollection', features: [] });
                } else {
                    this.map.getSource('hecate-delta').setData(this.delta.features);

                    this.map_default_unstyle();

                    this.delta.bbox = turf.bbox(this.delta.features);
                    this.map.fitBounds(this.delta.bbox);
                }
            }
        },
        methods: {
            logout: function() {
                this.credentials.authed = false;
                document.cookie = 'session=; Max-Age=0'
            },
            login: function() {
                fetch(`http://${window.location.host}/api/user/session`, {
                    method: 'GET',
                    credentials: 'same-origin',
                    headers: new Headers({
                        'Authorization': 'Basic '+ btoa(`${window.vue.modal.login.username}:${window.vue.modal.login.password}`)
                    })
                }).then((body) => {
                    this.modal.login = {
                        username: '',
                        password: ''
                    };
                    this.modal.type = false;
                    this.credentials.authed = true;
                });

            }, login_show: function() {
                if (!this.credentials.authed) this.modal.type = 'login';
            },
            register: function() {
                fetch(`http://${window.location.host}/api/user/create?username=${this.modal.register.username}&password=${this.modal.register.password}&email=${this.modal.register.email}`).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.modal.register = {
                        username: '',
                        password: '',
                        email: ''
                    };

                    this.modal.ok = {
                        header: 'Account Created',
                        body: 'Your account has been created! You can login now'
                    }
                    this.modal.type = 'ok'
                });
            },
            register_show: function() {
                this.modal.type = 'register';
            },
            bounds_refresh: function() {
                fetch(`http://${window.location.host}/api/data/bounds`).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.bounds = body;
                });
            },
            deltas_refresh: function() {
                fetch(`http://${window.location.host}/api/deltas`).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.deltas.splice(0, this.deltas.length);
                    this.deltas = this.deltas.concat(body);
                });
            },
            delta_get: function(delta_id) {
                if (!delta_id) return;

                fetch(`http://${window.location.host}/api/delta/${delta_id}`).then((response) => {
                      return response.json();
                }).then((body) => {
                    body.features.features = body.features.features.map(feat => {
                        feat.properties._action = feat.action;
                        return feat;
                    });
                    this.delta = body;
                });
            },
            feature_get: function(feature_id) {
                if (!feature_id) return;

                fetch(`http://${window.location.host}/api/data/feature/${feature_id}`).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.feature = body;
                });
            },
            map_delta_style: function() {
                let action_create = '#008000';
                let action_modify = '#FFFF00';
                let action_delete = '#FF0000';
                
                this.map.addLayer({
                    id: 'hecate-delta-polygons',
                    type: 'fill',
                    source: 'hecate-delta',
                    filter: ['==', '$type', 'Polygon'],
                    paint: {
                        'fill-opacity': 0.4,
                        'fill-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ]
                    }
                });

                this.map.addLayer({
                    id: 'hecate-delta-polygon-outlines',
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
                    id: 'hecate-delta-lines',
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
                    id: 'hecate-delta-points',
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
        maxzoom: 14,
        zoom: 3
    });

    window.vue.map.on('load', () => {
        window.vue.map.addSource('hecate-data', {
            type: 'vector',
            tiles: [ `http://${window.location.host}/api/tiles/{z}/{x}/{y}` ]
        });

        window.vue.map.addSource('hecate-delta', {
            type: 'geojson',
            data: { type: 'FeatureCollection', features: [] }
        });

        window.vue.map_default_style();
        window.vue.map_delta_style();
    });

    window.vue.map.addControl(new MapboxGeocoder({
        accessToken: mapboxgl.accessToken,
    }));

    window.vue.map.on('click', (e) => {
        if (window.vue.delta) return; //Don't currently support showing features within a delta

        let clicked = window.vue.map.queryRenderedFeatures(e.point)[0];

        if (clicked && clicked.properties.id) window.vue.feature_get(clicked.properties.id);
    });
}

