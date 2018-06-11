<template>
    <div class='viewport-full relative scroll-hidden'>
        <!-- Map -->
        <div id="map" class='h-full bg-darken10 viewport-twothirds viewport-full-ml absolute top left right bottom'></div>

        <!-- Left Panel -->
        <div class='absolute top-ml left bottom z1 w-full w240-ml hmax-full px12 py12-ml' style="pointer-events: none;">

            <!-- Toolbar -->
            <div class='bg-white round mb12' style='height: 40px; pointer-events:auto;'>
                <div @click="panel === 'deltas' ? panel = false : panel = 'deltas'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-clock'/></svg>
                </div>
                <div @click="panel === 'styles' ? panel = false : panel = 'styles'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-paint'/></svg>
                </div>
                <div @click="modal.type = 'query'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-inspect'/></svg>
                </div>
                <div @click="panel === 'bounds' ? panel = false : panel = 'bounds'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-arrow-down'/></svg>
                </div>
                <div @click="modal.type = 'settings'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-sprocket'/></svg>
                </div>
            </div>

            <template v-if='panel === "deltas"'><deltas :map='map'/></template>
            <template v-else-if='panel === "bounds"'><bounds/></template>
            <template v-else-if='panel === "styles"'><styles :credentials='credentials'/></template>
            <!--<feature :panel='panel'/>-->
        </div>

        <!-- Modal Opaque -->
        <div v-if='modal.type' class='absolute top left bottom right z2 bg-black opacity75' style="pointer-events: none;"></div>

        <!--Modals here-->

        <div v-if='modal.type === "register"' class='absolute top left bottom right z3' style="pointer-events: none;">
            <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
                <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
                    <div class='grid w-full'>
                        <div class='col col--8'>
                            <h3 class='fl py6 txt-m txt-bold w-full'>Hecate Register</h3>
                        </div>
                        <div class='col col--4'>
                            <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                        </div>

                        <div class='col col--12 py12'>
                            <label>
                                Username:
                                <input v-model="modal.register.username" class='input py12' placeholder='username'/>
                            </label>
                        </div>

                        <div class='col col--12 py12'>
                            <label>
                                Email:
                                <input v-model='modal.register.email' class='input py12' placeholder='email'/>
                            </label>
                        </div>

                        <div class='col col--12 py12'>
                            <label>
                                Password:
                                <input v-model='modal.register.password' type='password' class='input py12' placeholder='password'/>
                            </label>
                        </div>

                        <div class='col col--12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6 py12'>
                                    <button @click='modal.type = false' class='btn round bg-gray w-full'>Cancel</button>
                                </div>

                                <div class='col col--6 py12'>
                                    <button @click='register' class='btn round w-full'>Register</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div v-if='modal.type === "style_set"' class='absolute top left bottom right z3' style="pointer-events: none;">
            <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
                <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
                    <div class='grid w-full'>
                        <div class='col col--11'>
                            <template v-if="credentials.uid === modal.style_set.uid">
                                <input class='input my12' v-model="modal.style_set.name" placeholder="Style Name"/>
                            </template>
                            <template v-else>
                                <h3 class='fl py6 txt-m txt-bold fl'><span v-text='`${modal.style_set.username}/${modal.style_set.name}`'></span></h3>
                            </template>
                        </div>

                        <div class='col col--1'>
                            <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                        </div>

                        <div class='col col--12'>
                            <textarea :readonly="credentials.uid !== modal.style_set.uid" class='textarea w-full h360' v-model="modal.style_set.style" placeholder="Style JSON"></textarea>
                        </div>

                        <div class='col col--12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--8 py12'>
                                    <template v-if="credentials.authed && credentials.uid !== modal.style_set.uid">
                                        <button @click='style_create(modal.style_set.style, modal.style_set.name)' class='btn round btn--stroke w-full'>Clone &amp; Edit</button>
                                    </template>
                                    <template v-else-if="credentials.authed && modal.style_set.id">
                                        <label class='switch-container my6'>
                                            <input type='checkbox' v-model="modal.style_set.public"/>
                                            <div class='switch mr6'></div>
                                            Public Style
                                        </label>
                                    </template>
                                </div>

                                <div class='col col--4 py12'>
                                    <template v-if="credentials.uid === modal.style_set.uid">
                                        <button @click='style_update(modal.style_set.id, modal.style_set.name, JSON.parse(modal.style_set.style))' class='btn round btn--stroke w-full'>Save &amp; Apply Style</button>
                                    </template>
                                    <template v-else>
                                        <button @click='style_set(modal.style_set.id, JSON.parse(modal.style_set.style))' class='btn round btn--stroke w-full'>Apply Style</button>
                                    </template>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div v-if='modal.type === "ok"' class='absolute top left bottom right z3' style="pointer-events: none;">
            <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
                <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
                    <div class='grid w-full'>
                        <div class='col col--8'>
                            <h3 class='fl py6 txt-m txt-bold w-full' v-text='modal.ok.header'></h3>
                        </div>
                        <div class='col col--4'>
                            <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                        </div>

                        <div class='col col--12 py24' v-text='modal.ok.body'></div>

                        <div class='col col--12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6 py12'></div>

                                <div class='col col--6 py12'>
                                    <button @click='modal.type = false' class='btn round w-full'>OK</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div v-if='modal.type === "query"' class='absolute top left bottom right z3' style="pointer-events: none;">
            <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
                <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
                    <div class='grid w-full'>
                        <template v-if='!modal.query.results.length'>
                            <div class='pb12 col col--11'>
                                <h3 class='fl txt-m txt-bold fl'>SQL Query Editor</h3>
                            </div>

                            <div class='col col--1'>
                                <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                            </div>

                            <div class='col col--12'>
                                <textarea :readonly="!credentials.uid" class='textarea w-full h360' v-model="modal.query.query" placeholder="Query SQL"></textarea>
                            </div>

                            <div class='col col--12'>
                                <p>Note the web UI only supports querying up to 100 features</p>
                            </div>
                            <div class='col col--12'>
                                <div class='grid grid--gut12'>
                                    <div class='col col--8 py12'></div>
                                    <div class='col col--4 py12'>
                                        <button @click="query(modal.query.query)" class='btn round btn--stroke w-full'>Query</button>
                                    </div>
                                </div>
                            </div>
                        </template>
                        <template v-else>
                            <div class='pb12 col col--11'>
                                <button @click="modal.query.results = ''" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
                                <h3 class='fl pl12 txt-m txt-bold fl'>SQL Results Viewer</h3>
                            </div>

                            <div class='col col--1'>
                                <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                            </div>

                            <div class='col col--12'>
                                <textarea readonly class='textarea w-full h360' v-model="modal.query.results" placeholder="SQL Results"></textarea>
                            </div>

                            <div class='col col--12'>
                                <div class='grid grid--gut12'>
                                    <div class='col col--8 py12'></div>
                                    <div class='col col--4 py12'>
                                        <button @click="modal.type = false" class='btn round btn--stroke w-full'>Close</button>
                                    </div>
                                </div>
                            </div>
                        </template>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script>
// === Components ===
import Foot from './components/Foot.vue';

// === Panels ===
import Deltas from './panels/Deltas.vue';
import Feature from './panels/Feature.vue';
import Bounds from './panels/Bounds.vue';
import Styles from './panels/Styles.vue';

// === Modals ===

export default {
    name: 'app',
    data: function() {
        return {
            credentials: {
                map: { key: 'pk.eyJ1Ijoic2JtYTQ0IiwiYSI6ImNpcXNycTNqaTAwMDdmcG5seDBoYjVkZGcifQ.ZVIe6sjh0QGeMsHpBvlsEA' },
                authed: false,
                username: '',
                uid: false
            },
            map: {
                gl: false,
                layers: [],
                default: function() {
                    const foregroundColor = '#FF0000';

                    this.layers.push('hecate-data-polygons');
                    this.gl.addLayer({
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
                    this.gl.addLayer({
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
                    this.gl.addLayer({
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
                    this.gl.addLayer({
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
                },
                unstyle: function() {
                    for (let layer of this.layers) {
                        this.gl.removeLayer(layer);
                    }
                    this.layers = [];
                }
            },
            panel: false, //Store the current panel view (Deltas, Styles, Bounds, etc)
            feature: false, //Store the currently selected feature - overides panel view
            layers: [], //Store list of GL layer names so they can be easily removed
            style: false, //Store the id of the current style - false for generic style
            modal: {
                type: false,
                settings: {
                    header: '',
                    body: ''
                },
                error: {
                    header: '',
                    body: ''
                },
                query: {
                    query: '',
                    results: []
                },
                ok: {
                    header: '',
                    body: ''
                },
                style_set: {
                    style: '',
                    username: '',
                    uid: false,
                    public: false,
                    name: ''
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
        }
    },
    components: {
        foot: Foot,
        deltas: Deltas,
        bounds: Bounds,
        feature: Feature,
        styles: Styles
    },
    created: function() {
        this.logout();
    },
    watch: {
        panel: function() {
            this.refresh();
        }
    },
    mounted: function(e) {
        mapboxgl.accessToken = this.credentials.map.key;
        this.map.gl = new mapboxgl.Map({
            container: 'map',
            style: 'mapbox://styles/mapbox/satellite-v9',
            center: [-96, 37.8],
            hash: true,
            maxzoom: 14,
            zoom: 3
        });

        this.map.gl.on('load', () => {
            this.map.gl.addSource('hecate-data', {
                type: 'vector',
                maxzoom: 14,
                tiles: [ `http://${window.location.host}/api/tiles/{z}/{x}/{y}` ]
            });

            this.map.gl.addSource('hecate-delta', {
                type: 'geojson',
                data: { type: 'FeatureCollection', features: [] }
            });

            this.map.default();
        });

        this.map.gl.addControl(new MapboxGeocoder({
            accessToken: mapboxgl.accessToken,
        }));

        this.map.gl.on('click', (e) => {
            if (this.delta) return; //Don't currently support showing features within a delta

            let clicked = this.map.gl.queryRenderedFeatures(e.point)[0];

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
            document.cookie = 'session=;expires=Thu, 01 Jan 1970 00:00:01 GMT;'
            this.refresh();
        },
        refresh: function() {
            if (this.panel === 'Styles') {
                this.styles_refresh();
            }
        },
        login: function() {
            fetch(`http://${window.location.host}/api/user/session`, {
                method: 'GET',
                credentials: 'same-origin',
                headers: new Headers({
                    'Authorization': 'Basic '+ btoa(`${this.modal.login.username}:${this.modal.login.password}`)
                })
            }).then((response) => {
                if (response.status === 200) {
                    response.json().then((response) => {
                        this.modal.type = false;

                        this.credentials.authed = true;
                        this.credentials.username = this.modal.login.username;
                        this.credentials.uid = parseInt(response);
                        this.modal.login = {
                            username: '',
                            password: ''
                        };

                        this.refresh(); //Refresh the current panel as loggin in may reveal private data/styles
                    });
                } else {
                    return this.ok('Failed to Login', 'Failed to login with given credentials');
                }
            }).catch((err) => {
                return this.ok('Failed to Login', 'Failed to login with given credentials');
            });

        },
        query: function(query) {
            fetch(`http://${window.location.host}/api/data/query?limit=100&query=${encodeURIComponent(this.modal.query.query.replace(/;/g, ''))}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                  return response.text();
            }).then((body) => {
                this.modal.query.results = body;
            });
        },
        login_show: function() {
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
        styles_refresh: function() {
            fetch(`http://${window.location.host}/api/styles`).then((response) => {
                  return response.json();
            }).then((body) => {
                this.styles = body;
            });

            if (this.credentials.authed) {
                fetch(`http://${window.location.host}/api/styles/${this.credentials.uid}`, {
                    credentials: 'same-origin'
                }).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.pstyles = body.filter((style) => {
                        if (style.public) return false;
                        return true;
                    });
                });
            } else {
                this.pstyles = [];
            }
        },
        style_get: function(style_id, cb) {
            fetch(`http://${window.location.host}/api/style/${style_id}`, {
                credentials: 'same-origin'
            }).then((response) => {
                  return response.json();
            }).then((body) => {
                return cb(null, body);
            }).catch((err) => {
                return cb(err);
            });
        },
        style_create: function(style, name) {
            this.modal.style_set.style = style ? style : '';
            this.modal.style_set.id = false;
            this.modal.style_set.username = this.credentials.username;
            this.modal.style_set.uid = this.credentials.uid;
            this.modal.style_set.name = name ? `Copy of ${name}` : '';
            this.modal.style_set.public = false;

            this.modal.type = 'style_set';
        },
        style_update: function(style_id, style_name, style) {
            if (!style_id) { //Create new style
                fetch(`http://${window.location.host}/api/style`, {
                    method: 'POST',
                    credentials: 'same-origin',
                    headers: new Headers({
                        'Content-Type': 'application/json'
                    }),
                    body: JSON.stringify({
                        name: style_name,
                        style: style
                    })
                }).then((response) => {
                    if (response.status === 200) {
                        this.refresh();
                        this.style_set(style_id, style);
                    } else {
                        return this.ok('Failed to push style', 'Failed to update style');
                    }
                }).catch((err) => {
                    return this.ok('Failed to push style', 'Failed to update style');
                });
            } else { //Update Existing Style
                fetch(`http://${window.location.host}/api/style/${style_id}`, {
                    method: 'PATCH',
                    credentials: 'same-origin',
                    headers: new Headers({
                        'Content-Type': 'application/json'
                    }),
                    body: JSON.stringify({
                        name: style_name,
                        style: style
                    })
                }).then((response) => {
                    if (response.status !== 200) return this.ok('Failed to push style', 'Failed to update style');

                    if (this.credentials.authed && this.modal.style_set.id) {
                        fetch(`http://${window.location.host}/api/style/${style_id}/${this.modal.style_set.public ? 'public' : 'private'}`, {
                            method: 'POST',
                            credentials: 'same-origin'
                        }).then((response) => {
                            if (response.status !== 200) return this.ok('Failed to push style', 'Failed to update style');

                            this.refresh();
                            this.style_set(style_id, style);
                        }).catch((err) => {
                            console.error(err);
                            return this.ok('Failed to push style', 'Failed to update style');
                        });
                    } else {
                        this.refresh();
                        this.style_set(style_id, style);
                    }
                }).catch((err) => {
                    console.error(err);
                    return this.ok('Failed to push style', 'Failed to update style');
                });
            
            }
        },
        style_set: function(style_id, style) {
            if (!style.version || style.version !== 8) return this.ok('Style Not Applied', 'The selected style could not be applied. The style version must be 8');
            if (!style.layers || style.layers.length === 0) return this.ok('Style Not Applied', 'The selected style could not be applied. The style must contain at least 1 layer');

            this.map.unstyle();

            for (let layer of style.layers) {
                if (!layer.id) {
                    this.map.unstyle();
                    this.map.default();
                    return this.ok('Style Not Applied', 'Every layer in the style must have a unique id');
                }

                layer.source = 'hecate-data';
                layer['source-layer'] = 'data',

                this.map.layers.push(layer.id);
                this.map.gl.addLayer(layer);
            }

            this.modal.type = false;
        },
        style_set_modal: function(style_id) {
            this.style_get(style_id, (err, style) => {
                if (err) return this.ok('Failed to retrieve style', err.message);

                this.modal.style_set.style = JSON.stringify(style.style, null, 4);
                this.modal.style_set.id = style.id;
                this.modal.style_set.username = style.username;
                this.modal.style_set.uid = style.uid;
                this.modal.style_set.name = style.name;
                this.modal.style_set.public = style.public;

                this.modal.type = 'style_set';
            });
        },
        feature_get: function(feature_id) {
            if (!feature_id) return;

            fetch(`http://${window.location.host}/api/data/feature/${feature_id}`).then((response) => {
                  return response.json();
            }).then((body) => {
                this.feature = body;
            });
        }
    }
}
</script>
