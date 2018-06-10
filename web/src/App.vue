<template>
    <div class='viewport-full relative scroll-hidden'>
        <!-- Map -->
        <div id="map" class='h-full bg-darken10 viewport-twothirds viewport-full-ml absolute top left right bottom'></div>

        <!-- Left Panel -->
        <div class='absolute top-ml left bottom z1 w-full w240-ml hmax-full px12 py12-ml' style="pointer-events: none;">

            <!-- Toolbar -->
            <div class='bg-white round mb12' style='height: 40px; pointer-events:auto;'>
                <div @click="modal.type = 'settings'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-sprocket'/></svg>
                </div>
                <div @click="modal.type = 'deltas'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-clock'/></svg>
                </div>
                <div @click="modal.type = 'styles'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-paint'/></svg>
                </div>
                <div @click="modal.type = 'query'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-inspect'/></svg>
                </div>
                <div @click="modal.type = 'download'"class='py12 bg-white bg-darken25-on-hover btn round color-gray-dark cursor-pointer' style='height: 40px; width: 40px;'>
                    <svg class='icon'><use href='#icon-arrow-down'/></svg>
                </div>
            </div>

            <div v-if="feature" class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
                <heading :is_authed='credentials.authed' :login_show='login_show' :logout='logout' title='Hecate Deltas'></heading>

                <div class='flex-child clearfix px12 py12 bg-gray-faint round-b-ml txt-s'>
                    <button @click="feature = false" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
                    <div class="px12 py3 txt-bold fl">Feature <span v-text="feature.id"></span></div>
                </div>

                <div class="flex-child scroll-auto">
                    <table class='table txt-xs'>
                        <thead><tr><th>Key</th><th>Value</th></tr></thead>
                        <tbody>
                            <tr v-for="prop in Object.keys(feature.properties)">
                                <td v-text="prop"></td>

                                <!-- element: (Array) -->
                                <td v-if="Array.isArray(feature.properties[prop])">
                                    <template v-for="element in feature.properties[prop]" style="border-bottom: dotted;">
                                        <!-- element: Array: (Object) -->
                                        <template v-if="typeof element === 'object' && !Array.isArray(element)">
                                            <tr v-for="key in Object.keys(element)">
                                                <td v-text="key"></td>
                                                <td v-text="element[key]"></td>
                                            </tr>
                                        </template>
                                        <!-- element: Array: (Array, String, Number) -->
                                        <template v-else>
                                            <td v-text="JSON.stringify(element)"></td>
                                        </template>

                                        <div style="border-bottom: solid #CBCBCB 1px;"></div>
                                    </template>
                                </td>

                                <!-- element: (Object, String, Number) -->
                                <td v-else v-text="JSON.stringify(feature.properties[prop])"></td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <foot/>
            </div>
            <div v-else-if="delta" class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
                <heading :is_authed='credentials.authed' :login_show='login_show' :logout='logout' title='Hecate Deltas'></heading>

                <div class='flex-child clearfix px12 py12 bg-gray-faint round-b-ml txt-s'>
                    <button @click="delta = false" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
                    <div class="px12 py3 txt-bold fl">Delta <span v-text="delta.id"></span></div>
                </div>
                <div class="flex-child py12 px12 bg-gray-faint txt-s align-center"><span v-text="delta.props.message ? delta.props.message : '<No Delta Message>'"></span></div>

                <div class="flex-child scroll-auto">
                    <div v-for="(feat, feat_it) in delta.features.features" class="px12 py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                        <span v-if="feat.geometry.type === 'Point'" class="fl py6 px6"><svg class='icon'><use href='#icon-marker'/></span>
                        <span v-if="feat.geometry.type === 'MultiPoint'" class="fl px6 py6"><svg class='icon'><use href='#icon-marker'/></span>
                        <span v-if="feat.geometry.type === 'LineString'" class="fl px6 py6"><svg class='icon'><use href='#icon-polyline'/></span>
                        <span v-if="feat.geometry.type === 'MultiLineString'" class="fl px6 py6"><svg class='icon'><use href='#icon-polyline'/></span>
                        <span v-if="feat.geometry.type === 'Polygon'" class="fl px6 py6"><svg class='icon'><use href='#icon-polygon'/></span>
                        <span v-if="feat.geometry.type === 'MultiPolygon'" class="fl px6 py6"><svg class='icon'><use href='#icon-polygon'/></span>

                        <span class="fl" v-text="feat.id"></span>
                        <span class="fl px6" v-text="feat.action"></span>
                    </div>
                </div>

                <foot/>
            </div>
            <div v-else-if="panel == 'Bounds'" class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
                <heading :is_authed='credentials.authed' :login_show='login_show' :logout='logout' title='Hecate Bounds'></heading>

                <div class='flex-child px12 py12 bg-gray-faint round-b-ml txt-s'>
                    <template><panel v-model="panel"/></template>
                    <button @click="bounds_refresh" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>
                </div>

                <div class="flex-child scroll-auto">
                    <div v-if="!bounds.length" class="px12 py3 clearfix bg-white">
                        <div align="center">No Boundaries</div>
                    </div>

                    <div v-for="(bound, bound_it) in bounds" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                        <a v-bind:href="`/api/data/bounds/${bound}`" target="_blank" class="w-full clearfix">
                            <span class="fl py6 px6"><svg class='icon'><use href='#icon-database'/></span>
                            <span class="fl" v-text="bound"></span>
                        </a>
                    </div>
                </div>

                <foot/>
            </div>
            <div v-else-if="panel == 'Styles'" class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
                <heading :is_authed='credentials.authed' :login_show='login_show' :logout='logout' title='Hecate Styles'></heading>

                <div class='flex-child px12 py12 bg-gray-faint round-b-ml txt-s'>
                    <template><panel v-model="panel"/></template>
                    <button @click="styles_refresh" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>

                    <template v-if="credentials.authed">
                        <button @click="style_create()" class='fr btn mx6 btn--s round align-center'>New</button>
                    </template>
                </div>

                <div class="flex-child scroll-auto">
                    <div v-if="!styles.length" class="px12 py3 clearfix bg-white">
                        <div align="center">No Custom Styles</div>
                    </div>

                    <div v-for="(style, style_it) in pstyles" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                        <div @click="style_set_modal(style.id)" class="w-full clearfix">
                            <span class="fl py6 px6"><svg class='icon'><use href='#icon-lock'/></span>
                            <div class="fl">
                                <span v-text="style.username"></span>/<span v-text="style.name"></span>
                            </div>
                        </div>
                    </div>

                    <div v-for="(style, style_it) in styles" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                        <div @click="style_set_modal(style.id)" class="w-full clearfix">
                            <span class="fl py6 px6"><svg class='icon'><use href='#icon-paint'/></span>
                            <div class="fl">
                                <span v-text="style.username"></span>/<span v-text="style.name"></span>
                            </div>
                        </div>
                    </div>
                </div>

                <foot/>
            </div>
            <div v-else class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
                <heading :is_authed='credentials.authed' :login_show='login_show' :logout='logout' title='Hecate Deltas'></heading>

                <div class='flex-child px12 py12 bg-gray-faint round-b-ml txt-s'>
                    <template><panel v-model="panel"/></template>
                    <button @click="deltas_refresh" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>
                </div>

                <div v-if="!deltas.length" class="px12 py3 clearfix bg-white">
                    <div align="center">No Deltas</div>
                </div>

                <div class="flex-child scroll-auto">
                    <div v-for="delta in deltas" @click="delta_get(delta.id)" class="px12 py12 border-b bg-darken10-on-hover border--gray-light cursor-pointer wfull">
                        <div class="clearfix">
                            <div class="fl txt-bold" v-text="delta.username"></div>
                            <div class="fr txt-em" v-text="moment(delta.created).add(moment().utcOffset(), 'minutes').fromNow()"></div>
                        </div>
                        <span v-text="delta.props.message ? delta.props.message : '<No Delta Message>'"></span>
                        <span class='bg-blue-faint color-blue inline-block px6 py3 my3 my3 txt-xs txt-bold round fr' v-text="delta.id"></span>
                    </div>
                </div>

                <foot/>
            </div>

        </div>

        <!-- Modal Opaque -->
        <div v-if='modal.type' class='absolute top left bottom right z2 bg-black opacity75' style="pointer-events: none;"></div>

        <!-- MODALS -->
        <div v-if='modal.type === "login"' class='absolute top left bottom right z3' style="pointer-events: none;">
            <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
                <div class="flex-child px12 py12 w600 h300 bg-white round-ml shadow-darken10">
                    <div class='grid w-full'>
                        <div class='col col--8'>
                            <h3 class='fl py6 txt-m txt-bold w-full'>Hecate Login</h3>
                        </div>
                        <div class='col col--4'>
                            <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                        </div>

                        <div class='col col--12 py12'>
                            <label>
                                Username:
                                <input v-model='modal.login.username' class='input py12' placeholder='username'/>
                            </label>
                        </div>

                        <div class='col col--12 py12'>
                            <label>
                                Password:
                                <input v-model='modal.login.password' type='password' class='input py12' placeholder='password'/>
                            </label>
                        </div>

                        <div class='col col--12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6 py12'>
                                    <button @click='register_show' class='btn round bg-gray w-full'>Register</button>
                                </div>

                                <div class='col col--6 py12'>
                                    <button @click='login' class='btn round w-full'>Login</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
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
import Heading from './Heading.vue';
import Panel from './Panel.vue';
import Foot from './Foot.vue';
import Moment from 'moment';

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
            panel: 'Deltas', //Store the current panel view (Deltas, Styles, Bounds, etc)
            feature: false, //Store the currently selected feature - overides panel view
            delta: false, //Store the currently selected delta - overrides panel view
            deltas: [], //Store a list of the most recent deltas
            bounds: [], //Store a list of all bounds
            pstyles: [], //If the user is authenticated, store a list of their private styles
            styles: [], //Store a list of public styles
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
        heading: Heading,
        panel: Panel,
        foot: Foot
    },
    created: function() {
        this.logout();
        this.deltas_refresh();
    },
    watch: {
        panel: function() {
            this.refresh();
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
        this.moment = Moment;

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
            document.cookie = 'session=;expires=Thu, 01 Jan 1970 00:00:01 GMT;'
            this.refresh();
        },
        refresh: function() {
            if (this.panel === 'Bounds') {
                this.bounds_refresh();
            } else if (this.panel === 'Styles') {
                this.styles_refresh();
            } else if (this.panel === 'Deltas') {
                this.deltas_refresh();
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
                this.modal.style_set.uid = style.uid;
                this.modal.style_set.name = style.name;
                this.modal.style_set.public = style.public;

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
}
</script>
