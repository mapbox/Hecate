<template>
    <div class='viewport-full relative scroll-hidden'>
        <!-- Map -->
        <div id="map" class='h-full bg-darken10 viewport-twothirds viewport-full-ml absolute top left right bottom'></div>

        <!-- Bottom BaseLayers -->
        <div class='absolute z1 h60 color-black' style='right: 40px; bottom: 10px;'>
            <template v-for='(layer, layer_idx) in map.baselayers' >
                <div @click='setBaseLayer(layer_idx)' class='w60 h60 fr bg-white mx6 round cursor-pointer bg-gray-light-on-hover'>
                    <div class='w-full pt3'>
                        <template v-if='layer.type === "Raster"'>
                            <svg class='icon mx-auto align-center' style='width: 15px;'><use href='#icon-paint'/></svg>
                        </template>
                        <template v-else>
                            <svg class='icon mx-auto' style='width: 15px;'><use href='#icon-satellite'/></svg>
                        </template>
                    </div>
                    <div class='w-full align-center txt-xs' v-text='layer.name'></div>
                </div>
            </template>
        </div>

        <!-- Left Panel -->
        <div class='absolute top-ml left bottom z1 w-full w240-ml hmax-full px12 py12-ml' style="pointer-events: none;">

            <!-- Toolbar -->
            <div class='bg-white round mb12' style='height: 40px; pointer-events:auto;'>
                <div @click="panel === 'deltas' ? panel = false : panel = 'deltas'" class='py12 bg-white bg-darken25-on-hover round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-clock'/></svg>
                </div>
                <div style='width: 4px' class='fl h-full'></div>
                <div @click="panel === 'styles' ? panel = false : panel = 'styles'" class='py12 bg-white bg-darken25-on-hover round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-paint'/></svg>
                </div>
                <div style='width: 4px' class='fl h-full'></div>
                <div @click="panel = false; modal = 'query'" class='py12 bg-white bg-darken25-on-hover round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-inspect'/></svg>
                </div>
                <div style='width: 4px' class='fl h-full'></div>
                <div @click="panel === 'bounds' ? panel = false : panel = 'bounds'" class='py12 bg-white bg-darken25-on-hover round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-arrow-down'/></svg>
                </div>
                <div style='width: 4px' class='fl h-full'></div>
                <div @click="panel = false; modal = 'settings'" class='py12 bg-white bg-darken25-on-hover round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-sprocket'/></svg>
                </div>
            </div>

            <template v-if='panel === "deltas"'>
                <deltas :map='map' v-on:user='modal = "user"; user_id = $event'/>
            </template>
            <template v-else-if='panel === "bounds"'>
                <bounds :map='map' :panel='panel'/>
            </template>
            <template v-else-if='panel === "styles"'>
                <styles :credentials='credentials' v-on:style='modal = "style"; style_id = $event'/>
            </template>
            <template v-else-if='feature'>
                <feature :map='map' :id='feature' v-on:close='feature = false'/>
            </template>
        </div>

        <!-- Login Panel -->
        <div class='none block-ml absolute top-ml left bottom z1 ml240 hmax-full py12-ml' style="pointer-events: none;">
            <div class='bg-white round' style='height: 40px; pointer-events:auto;'>
                <div @click="credentials.authed ? modal = 'self' : modal = 'login'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-user'/></svg>
                </div>
                <div v-if='credentials.authed' @click="logout(true)" class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer h-full px12 fl' style='width: 40px;'>
                    <svg class='icon'><use href='#icon-logout'/></svg>
                </div>
            </div>
        </div>

        <!-- Modal Opaque -->
        <div v-if='modal' class='absolute top left bottom right z2 bg-black opacity75' style="pointer-events: none;"></div>

        <!--Modals here-->
        <template v-if='modal === "login"'>
            <login
                v-on:login='modal = false; credentials.authed = true'
                v-on:close='modal = false'
                v-on:register='modal = "register"'
                v-on:username='credentials.username = $event'
                v-on:uid='credentials.uid = $event'
            />
        </template>
        <template v-else-if='modal === "register"'>
            <register v-on:close='modal = false' />
        </template>
        <template v-else-if='modal === "user"'>
            <user :auth='auth' :user='user_id' v-on:close='modal = false' />
        </template>
        <template v-else-if='modal === "self"'>
            <self :auth='auth' v-on:close='modal = false' />
        </template>
        <template v-else-if='modal === "settings"'>
            <settings v-on:close='settings_close' :auth='auth'/>
        </template>
        <template v-else-if='modal === "query"'>
            <query :auth='auth' v-on:close='modal = false' :credentials='credentials' />
        </template>
        <template v-else-if='modal === "style"'>
            <stylem v-on:close='modal = false' :style_id='style_id' :credentials='credentials' :map='map' />
        </template>
    </div>
</template>

<script>
//Libaries
import mapboxglgeo from '@mapbox/mapbox-gl-geocoder';

// === Components ===
import Foot from './components/Foot.vue';

// === Panels ===
import Deltas from './panels/Deltas.vue';
import Feature from './panels/Feature.vue';
import Bounds from './panels/Bounds.vue';
import Styles from './panels/Styles.vue';

// === Modals ===
import User from './modals/User.vue';
import Self from './modals/Self.vue';
import Login from './modals/Login.vue';
import Register from './modals/Register.vue';
import Settings from './modals/Settings.vue';
import Query from './modals/Query.vue';
import Style from './modals/Style.vue';

export default {
    name: 'app',
    data: function() {
        return {
            auth: false,
            credentials: {
                map: { key: 'pk.eyJ1IjoiaW5nYWxscyIsImEiOiJjam42YjhlMG4wNTdqM2ttbDg4dmh3YThmIn0.uNAoXsEXts4ljqf2rKWLQg' },
                authed: false,
                username: '',
                uid: false
            },
            map: {
                gl: false,
                baselayers: [],
                layers: [],
                defaults: [],
                default: function() {
                    this.gl.addSource('hecate-data', {
                        type: 'vector',
                        maxzoom: 17,
                        tiles: [ `${window.location.protocol}//${window.location.host}/api/tiles/{z}/{x}/{y}` ]
                    });

                    this.gl.addSource('hecate-delta', {
                        type: 'geojson',
                        data: { type: 'FeatureCollection', features: [] }
                    });

                    this.gl.addSource('hecate-bounds', {
                        type: 'geojson',
                        data: { type: 'FeatureCollection', features: [] }
                    });

                    this.gl.addLayer({
                        id: 'hecate-bounds',
                        type: 'fill',
                        source: 'hecate-bounds',
                        filter: ['==', '$type', 'Polygon'],
                        paint: {
                            'fill-opacity': 0.3,
                            'fill-color': '#FF0000'
                        }
                    });

                    const foregroundColor = '#FF0000';
                    this.defaults.push('hecate-data-polygons');
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

                    this.defaults.push('hecate-data-polygon-outlines');
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

                    this.defaults.push('hecate-data-lines');
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

                    this.defaults.push('hecate-data-points');
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

                /**
                 * Removes all user given styles from the map
                 */
                unstyle: function() {
                    for (let layer of this.layers) {
                        this.gl.removeLayer(layer);
                    }
                    this.layers = [];
                },

                /**
                 * Hide default style
                 */
                hide: function() {
                    for (let layer of this.defaults) this.gl.setLayoutProperty(layer, 'visibility', 'none');
                },

                /**
                 * Show default style
                 */
                show: function() {
                    for (let layer of this.defaults) this.gl.setLayoutProperty(layer, 'visibility', 'visible');
                }
            },
            panel: false, //Store the current panel view (Deltas, Styles, Bounds, etc)
            layers: [], //Store list of GL layer names so they can be easily removed
            feature: false, //Store the id of a clicked feature
            schema: false, //Store the JSON Schema for features
            user_id: false, //Store the id of a user for viewing more info
            style_id: false, //Store the id of the current style - false for generic style
            modal: false
        }
    },
    components: {
        self: Self,
        user: User,
        foot: Foot,
        deltas: Deltas,
        bounds: Bounds,
        feature: Feature,
        styles: Styles,
        login: Login,
        register: Register,
        settings: Settings,
        query: Query,
        stylem: Style
    },
    mounted: function(e) {
        mapboxgl.accessToken = this.credentials.map.key;

        this.getSelf((err, user) => {
            if (err || !user) {
                this.getLayers();
                this.getAuth();
            } else {
                this.credentials.authed = true;
                this.credentials.username = user.username;
                this.credentials.uid = user.id;

                this.getLayers();
                this.getAuth();
            }
        });

        this.map.gl = new mapboxgl.Map({
            container: 'map',
            attributionControl: false,
            style: 'mapbox://styles/mapbox/satellite-v9',
            center: [-96, 37.8],
            hash: true,
            maxzoom: 14,
            zoom: 3
        }).addControl(new mapboxgl.AttributionControl({
            compact: true
        })).addControl(new mapboxglgeo({
            accessToken: mapboxgl.accessToken,
        }));

        this.map.gl.on('style.load', () => {
            this.map.default();
        });

        this.map.gl.on('mouseenter', 'hecate-data-polygons',  (e) => {
            this.map.gl.getCanvas().style.cursor = 'pointer';
        });
        this.map.gl.on('mouseleave', 'hecate-data-polygons', (e) => {
            this.map.gl.getCanvas().style.cursor = '';
        });
        this.map.gl.on('mouseenter', 'hecate-data-lines',  (e) => {
            this.map.gl.getCanvas().style.cursor = 'pointer';
        });
        this.map.gl.on('mouseleave', 'hecate-data-lines', (e) => {
            this.map.gl.getCanvas().style.cursor = '';
        });
        this.map.gl.on('mouseenter', 'hecate-data-points',  (e) => {
            this.map.gl.getCanvas().style.cursor = 'pointer';
        });
        this.map.gl.on('mouseleave', 'hecate-data-points', (e) => {
            this.map.gl.getCanvas().style.cursor = '';
        });

        this.map.gl.on('click', (e) => {
            if (this.modal === 'delta') return; //Don't currently support showing features within a delta

            this.feature = [e.lngLat.lng, e.lngLat.lat];
        });
    },
    watch: {
        panel: function() {
            console.error(this.panel);
            if (this.panel !=='bounds') {
                this.map.gl.getSource('hecate-bounds').setData({
                    type: 'Feature',
                    features: []
                });
            }
        }
    },
    methods: {
        settings_close: function() {
            this.modal = false;
            this.getLayers();
        },
        getLayers: function() {
            window.hecate.meta.get('layers', (err, layers) => {
                if (err) console.error(err);

                if (!layers) return;
                this.map.baselayers = layers;
            });
        },
        getAuth: function() {
            window.hecate.auth.get((err, auth) => {
                if (err) console.error(err);
                this.auth = auth;
            });
        },
        setBaseLayer(layer_idx) {
            if (isNaN(layer_idx)) return;

            this.map.gl.setStyle(this.map.baselayers[layer_idx].url);
        },
        getSchema: function() {
            window.hecate.schema.get((err, schema) => {
                if (err) console.error(err);
                this.schema = schema;
            });
        },
        getSelf: function(cb) {
            fetch(`${window.location.protocol}//${window.location.host}/api/user/info`, {
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status === 401) {
                    return this.logout();
                } else if (response.status !== 200) {
                    return cb(new Error('not logged in'));
                } else {
                    return response.json();
                }
            }).then((user) => {
                if (cb) return cb(null, user);
            }).catch((err) => {
                console.error(err);

                if (cb) return cb(err);
            });
        },
        logout: function(reload) {
            this.credentials.authed = false;

            console.error('LOGOUT');
            fetch(`${window.location.protocol}//${window.location.host}/api/user/session`, {
                method: 'DELETE',
                credentials: 'same-origin'
            }).then((response) => {
                if (reload) window.location.reload();
            });
        }
    }
}
</script>
