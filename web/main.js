/*jshint browser:true,curly: false */
/* global L */

window.onload = () => {
    window.vue = new Vue({
        el: '#app',
        data: {
            credentials: {
                map: { key: 'pk.eyJ1Ijoic2JtYTQ0IiwiYSI6ImNpcXNycTNqaTAwMDdmcG5seDBoYjVkZGcifQ.ZVIe6sjh0QGeMsHpBvlsEA' },
                authed: false,
                username: ''
            },
            panel: 'Deltas', //Store the current panel view (Deltas, Styles, Bounds, etc)
            feature: false, //Store the currently selected feature - overides panel view
            delta: false, //Store the currently selected delta - overrides panel view
            deltas: [], //Store a list of the most recent deltas
            bounds: [], //Store a list of all bounds
            styles: [], //Store a list of public styles
            layers: [], //Store list of GL layer names so they can be easily removed
            style: false, //Store the id of the current style - false for generic style
            modal: {
                type: false,
                error: {
                    header: '',
                    body: ''
                },
                ok: {
                    header: '',
                    body: ''
                },
                style_set: {
                    style: '',
                    username: '',
                    uid: false,
                    name: ''
                },
                style_create: {
                    name: '',
                    style: ''
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
            }
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
                props: [ 'is_authed', 'login_show', 'logout', 'title' ]
            },
            panel: {
                data: function() {
                    return {
                        panel: 'Deltas'
                    };
                },
                watch: {
                    panel: function() {
                        this.$emit('input', this.panel);
                    }
                },
                template: `
                    <div class='select-container'>
                        <select v-model="panel" class='select select--s select--stroke'>
                            <option>Deltas</option>
                            <option>Bounds</option>
                            <option>Styles</option>
                        </select>
                        <div class='select-arrow'></div>
                    </div>
                `
            },
            foot: {
                template: `
                    <div class='flex-child px12 py12 bg-gray-faint round-b-ml txt-s flex-child'>
                        <div align='center'><a href="https://github.com/ingalls/hecate">Powered By Hecate Server</a></div>
                    </div>
                `
            }
        },
        created: function() {
            this.deltas_refresh();
            this.logout();
        },
        watch: {
            panel: function() {
                if (this.panel === 'Bounds') {
                    this.bounds_refresh();
                } else if (this.panel === 'Styles') {
                    this.styles_refresh();
                } else if (this.panel === 'Deltas') {
                    this.deltas_refresh();
                }
            },
            delta: function() {
                //Reset Normal Map
                if (!this.delta) {
                    this.map_unstyle();
                    this.map_default_style();

                    this.map.getSource('hecate-delta').setData({ type: 'FeatureCollection', features: [] });
                } else {
                    this.map.getSource('hecate-delta').setData(this.delta.features);
                    this.map_unstyle();
                    this.map_delta_style();

                    this.delta.bbox = turf.bbox(this.delta.features);
                    this.map.fitBounds(this.delta.bbox);
                }
            }
        },
        mounted: function(e) {
            mapboxgl.accessToken = this.credentials.map.key;
            this.map = new mapboxgl.Map({
                container: 'map',
                style: 'mapbox://styles/mapbox/satellite-v9',
                center: [-96, 37.8],
                hash: true,
                maxzoom: 14,
                zoom: 3
            });

            this.map.on('load', () => {
                this.map.addSource('hecate-data', {
                    type: 'vector',
                    maxzoom: 14,
                    tiles: [ `http://${window.location.host}/api/tiles/{z}/{x}/{y}` ]
                });

                this.map.addSource('hecate-delta', {
                    type: 'geojson',
                    data: { type: 'FeatureCollection', features: [] }
                });

                this.map_default_style();
            });

            this.map.addControl(new MapboxGeocoder({
                accessToken: mapboxgl.accessToken,
            }));

            this.map.on('click', (e) => {
                if (this.delta) return; //Don't currently support showing features within a delta

                let clicked = this.map.queryRenderedFeatures(e.point)[0];

                if (clicked && clicked.properties['hecate:id']) this.feature_get(clicked.properties['hecate:id']);
            });
        },
        methods: {
            ok: function(header, body) {
                this.modal.ok.header = header;
                this.modal.ok.body = body;
                this.modal.type = 'ok';
            },
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
                }).then((response) => {
                    if (response.status === 200) {
                        this.modal.type = false;
                        this.credentials.authed = true;
                        this.credentials.username = this.modal.login.username;
                        this.modal.login = {
                            username: '',
                            password: ''
                        };
                    } else {
                        return this.ok('Failed to Login', 'Failed to login with given credentials');
                    }
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

                    this.ok('Account Created', 'Your account has been created! You can login now');
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
            styles_refresh: function() {
                fetch(`http://${window.location.host}/api/styles`).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.styles = body;
                });
            },
            style_get: function(style_id, cb) {
                fetch(`http://${window.location.host}/api/style/1`).then((response) => {
                      return response.json();
                }).then((body) => {
                    return cb(null, body);
                }).catch((err) => {
                    return cb(err);
                });
            },
            style_set: function(style_id, style) {
                if (!style.version || style.version !== 8) return this.ok('Style Not Applied', 'The selected style could not be applied. The style version must be 8');
                if (!style.layers || style.layers.length === 0) return this.ok('Style Not Applied', 'The selected style could not be applied. The style must contain at least 1 layer');

                this.map_unstyle();

                for (let layer of style.layers) {
                    if (!layer.id) {
                        this.map_unstyle();
                        this.map_default_style();
                        return this.ok('Style Not Applied', 'Every layer in the style must have a unique id');
                    }

                    layer.source = 'hecate-data';
                    layer['source-layer'] = 'data',

                    this.layers.push(layer.id);
                    this.map.addLayer(layer);
                }

                this.modal.type = false;
            },
            style_set_modal: function(style_id) {
                this.style_get(style_id, (err, style) => {
                    if (err) return this.ok('Failed to retrieve style', err.message);

                    this.modal.style_set.style = JSON.stringify(style.style, null, 4);
                    this.modal.style_set.id = style.id;
                    this.modal.style_set.username = style.username;
                    this.modal.style_set.name = style.name;

                    this.modal.type = 'style_set';
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
               
                this.layers.push('hecate-delta-polygons');
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

                this.layers.push('hecate-delta-polygon-outlines');
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

                this.layers.push('hecate-delta-lines');
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

                this.layers.push('hecate-delta-points');
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
            map_unstyle: function() {
                for (let layer of this.layers) {
                    this.map.removeLayer(layer);
                }
                this.layers = [];
            },
            map_default_style: function() {
                const foregroundColor = '#FF0000';

                this.layers.push('hecate-data-polygons');
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

                this.layers.push('hecate-data-polygon-outlines');
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

                this.layers.push('hecate-data-lines');
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

                this.layers.push('hecate-data-points');
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
}

