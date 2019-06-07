<template>
    <div class='absolute top left bottom right z3' style="pointer-events: none;">
        <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events: auto;">
            <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
                <template v-if='error'>
                    <div class='grid w-full col'>
                        <div class='col--12'>
                            <h3 class='w-full py6 txt-m txt-bold align-center'>ERROR!</h3>
                        </div>

                        <div class='col--12 py12' v-text='error'></div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'></div>
                                <div class='col col--6'>
                                    <button @click='error = false' class='btn round w-full'>Ok</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else-if='mode === "addLayer"'>
                    <div class='grid w-full col'>

                        <template v-if='layerData.error'>
                            <div class='col--12 color-white px12 bg-red round align-center'>
                                <h3 class='w-full py6 txt-m txt-bold' v-text='layerData.error'></h3>
                            </div>
                        </template>

                        <div class='col--12'>
                            <template v-if='layerData.id === false'>
                                <h3 class='w-full py6 txt-m txt-bold'>Add A New Base Layer</h3>
                            </template>
                            <template v-else>
                                <h3 class='w-full py6 txt-m txt-bold' v-text='"Modify the " + layerData.name + " layer"'></h3>
                            </template>
                        </div>

                        <div class='col--12 py12 px12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'>
                                    <label>Layer Name</label>
                                    <input v-model='layerData.name' class='input' placeholder='Layer Name' v-bind:class="{ 'input--border-red': layerData.nameError }"/>
                                </div>
                                <div class='col col--6'>
                                    <label >Layer Type</label>
                                    <div class='select-container w-full'>
                                        <select v-model='layerData.type' class='select' v-bind:class="{ 'input--border-red': layerData.typeError }">
                                            <option>Vector</option>
                                            <option>Raster</option>
                                        </select>
                                        <div class='select-arrow'></div>
                                    </div>
                                </div>
                                <div class='col col--12 py12'>
                                    <label>Mapbox:// Style</label>
                                    <input v-model='layerData.url' class='input w-full' placeholder='mapbox://' v-bind:class="{ 'input--border-red': layerData.urlError }" />
                                </div>
                            </div>
                        </div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'>
                                    <button @click='close' class='btn btn--red round w-full'>Cancel</button>
                                </div>
                                <div class='col col--6'>
                                    <template v-if='layerData.id === false'>
                                        <button @click='addLayer' class='btn round w-full'>Create Layer</button>
                                    </template>
                                    <template v-else>
                                        <button @click='updateLayer(layerData.id)' class='btn round w-full'>Update Layer</button>
                                    </template>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else-if='mode === "addHook"'>
                    <div class='grid w-full col'>

                        <template v-if='webhookData.error'>
                            <div class='col--12 color-white px12 bg-red round align-center'>
                                <h3 class='w-full py6 txt-m txt-bold' v-text='webhookData.error'></h3>
                            </div>
                        </template>

                        <div class='col--12'>
                            <template v-if='webhookData.id === false'>
                                <h3 class='w-full py6 txt-m txt-bold'>Add A New Webhook</h3>
                            </template>
                            <template v-else>
                                <h3 class='w-full py6 txt-m txt-bold' v-text='"Modify the " + webhookData.name + " webhook"'></h3>
                            </template>
                        </div>

                        <div class='col--12 py12 px12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--12 pb12'>
                                    <label>Webhook Name</label>
                                    <input v-model='webhookData.name' class='input' placeholder='Webhook Name' v-bind:class="{ 'input--border-red': webhookData.nameError }"/>
                                </div>
                                <div class='col col--12 pb12'>
                                    <label >Webhook Actions</label>
                                    <div class='col col--6'>
                                        <label class='w-full checkbox-container'>
                                            <input type='checkbox' />
                                            <div class='checkbox mr6'><svg class='icon'><use xlink:href='#icon-check' /></svg></div>
                                            User
                                        </label>
                                        <label class='w-full checkbox-container'>
                                            <input type='checkbox' />
                                            <div class='checkbox mr6'><svg class='icon'><use xlink:href='#icon-check' /></svg></div>
                                            Delta
                                        </label>
                                        <label class='w-full checkbox-container'>
                                            <input type='checkbox' />
                                            <div class='checkbox mr6'><svg class='icon'><use xlink:href='#icon-check' /></svg></div>
                                            Meta
                                        </label>
                                        <label class='w-full checkbox-container'>
                                            <input type='checkbox' />
                                            <div class='checkbox mr6'><svg class='icon'><use xlink:href='#icon-check' /></svg></div>
                                            Style
                                        </label>
                                    </div>
                                </div>
                                <div class='col col--12 pb12'>
                                    <label>Webhook URL</label>
                                    <input v-model='webhookData.url' class='input w-full' placeholder='https://' v-bind:class="{ 'input--border-red': webhookData.urlError }" />
                                </div>
                            </div>
                        </div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'>
                                    <button @click='close' class='btn btn--red round w-full'>Cancel</button>
                                </div>
                                <div class='col col--6'>
                                    <template v-if='webhookData.id === false'>
                                        <button @click='addHook' class='btn round w-full'>Create Webhook</button>
                                    </template>
                                    <template v-else>
                                        <button @click='updateHook(webhookData.id)' class='btn round w-full'>Update Webhook</button>
                                    </template>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else-if='mode === "helpBase"'>
                    <div class='grid w-full col'>
                        <div class='col--12'>
                            <h3 class='w-full py6 txt-m txt-bold align-center'>Base Layers Help</h3>
                        </div>

                        <div class='col--12 py12'>
                            <p class='py3'>
                                Base Layers are the background map that the data in the Hecate instance is displayed on.
                            </p>
                            <p class='py3'>
                                Common options include Satellite Imagery, Street, Political Boundaries, etc.
                            </p>
                            <p class='py3'>
                                Hecate currently supports adding any mapbox:// prefixed map. See documentation on creating
                                a style url <a href="https://www.mapbox.com/help/define-style-url/">here</a>. Prebuilt
                                mapbox style URLs can be obtained <a href="https://www.mapbox.com/api-documentation/#styles">here</a>.
                            </p>
                        </div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'></div>
                                <div class='col col--6'>
                                    <button @click='close' class='btn round w-full'>Got It!</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else-if='mode === "delLayer"'>
                    <div class='grid w-full col'>
                        <div class='col--12'>
                            <h3 class='w-full py6 txt-m txt-bold align-center'>Are you sure you want to delete this layer?</h3>

                            <div class='w120 mx-auto relative color-gray-light my12'>
                                <div class='w-full h120 round border border--gray-light'>
                                    <template v-if='delLayerData.type === "Raster"'>
                                        <svg class='icon w-full h-full'><use href='#icon-satellite'/></svg>
                                    </template>
                                    <template v-else>
                                        <svg class='icon w-full h-full'><use href='#icon-paint'/></svg>
                                    </template>
                                </div>

                                <div class='w-full color-black align-center' v-text='delLayerData.name'></div>
                            </div>
                        </div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'>
                                    <button @click='close' class='btn round w-full'>Cancel</button>
                                </div>
                                <div class='col col--6'>
                                    <button @click='deleteLayer' class='btn btn--red round w-full'>Delete</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else>
                    <div class='fl h360' style='width: 44px; padding-right: 8px;'>
                        <button @click='submode = "server"' class='fl btn btn--stroke round w36 px12 my6'>
                            <svg class='icon'><use href='#icon-sprocket'/></svg>
                        </button>

                        <button @click='submode = "users"' class='fl btn btn--stroke round w36 px12 my6'>
                            <svg class='icon'><use href='#icon-user'/></svg>
                        </button>

                        <button @click='submode = "webhooks"' class='fl btn btn--stroke round w36 px12 my6'>
                            <svg class='icon'><use href='#icon-link'/></svg>
                        </button>
                    </div>

                    <div class='fl pl6' style='width: calc(600px - 68px);'>
                        <template v-if='submode === "server"'>
                            <div class='col col--12 txt-m txt-bold'>
                                Server Settings
                                <button @click='close' class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                            </div>

                            <div class='py6 col col--12 border--gray-light border-b'>
                                <span class='txt-m'>Base Layers</span>
                                <span @click='helpBaseClick' class='fr cursor-pointer'><svg class='icon'><use href='#icon-info'/></svg></span>
                            </div>

                            <div class='col col--12 hmin120 hmax180 clearfix scroll-auto'>
                                <template v-for='(layer, layer_idx) of layers'>
                                    <div class='w120 fl relative color-gray-light my12 mx12 cursor-pointer'>
                                        <div @click='delLayerClick(layer_idx)' class='absolute bg-red-light bg-red-on-hover round color-white w18 h18' style='top: -9px; right: -9px;'>
                                            <svg class='icon w-full pt3'><use href='#icon-close'/></svg>
                                        </div>

                                        <div @click='editLayerClick(layer_idx)'  class='w-full h120 color-gray-on-hover round border border--gray-light border--gray-on-hover'>
                                            <template v-if='layer.type === "Raster"'>
                                                <svg class='icon w-full h-full'><use href='#icon-satellite'/></svg>
                                            </template>
                                            <template v-else>
                                                <svg class='icon w-full h-full'><use href='#icon-paint'/></svg>
                                            </template>
                                        </div>

                                        <div class='w-full color-black align-center' v-text='layer.name'></div>
                                    <div>
                                </template>

                                <!-- Add a new Base Layer -->
                                <div @click='newLayerClick' class='h120 w120 fl round border border--gray-light border--gray-on-hover color-gray-light color-gray-on-hover my12 mx12 cursor-pointer'>
                                    <svg class='icon w-full h-full'><use href='#icon-plus'/></svg>
                                </div>
                            </div>

                            <div class='py6 col col--12 border--gray-light border-b'>
                                <span class='txt-m'>Miscellaneous</span>
                            </div>

                            <div class='col col--12 my12'>
                                Wipe the cached vector tiles, forcing regen of all tiles
                                <button :disabled='tilecache' @click='clearCache' class='fr btn btn--red round'>
                                    <template v-if='tilecache'>Cache Cleared</template>
                                    <template v-else>Clear Cache</template>
                                </button>
                            </div>
                        </template>
                        <template v-if='submode === "users"'>
                            <div class='col col--12 txt-m txt-bold'>
                                User Settings
                                <button @click='close' class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                            </div>

                            <div class='py6 col col--12 border--gray-light border-b mb12'>
                                <span class='txt-m'>Users</span>
                            </div>

                            <div class='relative mb12'>
                                <div class='absolute flex-parent flex-parent--center-cross flex-parent--center-main w36 h36'><svg class='icon'><use href='#icon-search'></use></svg></div>
                                <input v-model='userFilter' class='input pl36' placeholder='Filter Users'>
                            </div>

                            <div class='col col--12 h240 scroll-auto'>
                                <template v-for='(user, user_idx) of users'>
                                    <div class='col col--12'>
                                       <div class='grid col h30 bg-gray-faint-on-hover cursor-pointer round'>
                                            <div class='col--2'>
                                                <span class='ml6 bg-blue-faint color-blue inline-block px6 py3 my3 my3 txt-xs txt-bold round' v-text="user.id"></span>
                                            </div>
                                            <div class='col--8' v-text='user.username'></div>
                                            <div class='col--2' v-text='user.access'></div>
                                        </div>
                                    </div>
                                </template>
                            </div>
                        </template>
                        <template v-if='submode === "webhooks"'>
                            <div class='col col--12 txt-m txt-bold'>
                                Webhook Settings
                                <button @click='close' class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                            </div>

                            <div class='py6 col col--12 border--gray-light border-b mb12'>
                                <span class='txt-m'>Webhooks</span>
                                <button @click="addHookClick" class='btn round h24 fr'>
                                    <svg class='icon h-full'><use href='#icon-plus'/></svg>
                                </button>
                            </div>

                            <template v-if="hooks.length">
                                <div class='col col--12 h240 scroll-auto'>
                                    <template v-for='(hook, hook_idx) of hooks'>
                                        <div class='col col--12'>
                                           <div class='grid col h30 bg-gray-faint-on-hover cursor-pointer round'>
                                                <span class="mx6" v-text='hook.name'></span>
                                                <template v-for='hook_action of hook.actions'>
                                                    <span class='bg-blue-faint color-blue px6 py3 my3 my3 txt-xs txt-bold round' v-text="hook_action"></span>
                                                </template>
                                           </div>
                                        </div>
                                    </template>
                                </div>
                            </template>
                            <template v-else>
                                <div class='col col--12 h240 scroll-auto'>
                                    <div class="align-center">No Webhooks Yet!</div>
                                </div>
                            </template>
                        </template>
                    </div>
                </template>
            </div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'settings',
    data: function() {
        return {
            mode: 'settings',
            submode: 'server',
            tilecache: false, //Store if the cache has been cleared or not
            hooks: [],
            layers: [],
            users: [],
            userFilter: '',
            error: '',
            delLayerData: {
                idx: false,
                name: '',
                type: ''
            },
            layerData: {
                id: false,
                error: '',
                name: '',
                nameError: false,
                type: '',
                typeError: false,
                url: '',
                urlError: false
            },
            webhookData: {
                id: false,
                error: '',
                name: '',
                nameError: false,
                url: '',
                urlError: false,
                actions: {
                    user: false,
                    delta: false,
                    style: false,
                    meta: false
                }
            }
        }
    },
    mounted: function() {
        this.getLayers();
        this.getUsers();
        this.getHooks();
    },
    watch: {
        userFilter: function() {
            this.getUsers();
        },
        error: function() {
            this.getLayers();
        }
    },
    methods: {
        close: function() {
            this.clearLayer();

            if (this.mode !== 'settings') {
                this.mode = 'settings';
            } else {
                this.$emit('close');
            }
        },
        clearCache: function() {
            window.hecate.tiles.clear((err) => {
                if (err) return this.error = err.message;
                this.tilecache = true;
            });
        },
        getLayers: function() {
            window.hecate.meta.get('layers', (err, layers) => {
                if (err) return this.error = err.message;
                this.layers = layers;
            });
        },
        getUsers: function() {
            window.hecate.users.list(this.userFilter, (err, users) => {
                if (err) return this.error = err.message;
                this.users = users;
            });
        },
        getHooks: function() {
            window.hecate.webhooks.list((err, hooks) => {
                if (err) return this.error = err.message;
                this.hooks = hooks;
            });
        },
        getHook: function(hook_id) {
            window.hecate.webhooks.get(hook_id, (err, hook) => {
                if (err) return this.error = err.message;

                this.webhookData.id = hook.id;
                this.webhookData.name = hook.name;
                this.webhookData.url = hook.url;

                // Ensure all checkboxes are false
                for (let check of Object.keys(this.webhookData.checkbox)) {
                    this.webhookData.checkbox[check] = false;
                }
                //Conditionally apply actions
                for (let action of this.webhookData.actions) {
                    this.webhookData.actions[action] = true;
                }
            });
        },
        putLayers: function() {
            fetch(`${window.location.protocol}//${window.location.host}/api/meta/layers`, {
                method: 'POST',
                credentials: 'same-origin',
                headers: new Headers({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(this.layers)
            }).then((response) => {
                if (response.status !== 200) {
                    this.error = response.status + ':' + response.statusText;
                }

                return response.json();
            }).then((layers) => {
                if (!layers)
                this.layers = layers;
            }).catch((err) => {
                this.error = err.message;
            });
        },
        deleteLayer: function() {
            if (isNaN(this.delLayerData.idx)) return;

            this.layers.splice(this.delLayerData.idx, 1);
            this.putLayers();

            this.close();
        },
        validateLayer: function() {
            let error = false;

            if (this.layerData.name.length === 0) {
                this.layerData.nameError = true;
                error = true;
            } else {
                this.layerData.nameError = false;
            }

            if (['Vector', 'Raster'].indexOf(this.layerData.type) === -1) {
                this.layerData.typeError = true;
                error = true;
            } else {
                this.layerData.typeError = false;
            }

            if (!this.layerData.url.match(/^mapbox:\/\//)) {
                this.layerData.urlError = true;
                error = true;
            } else {
                this.layerData.urlError = false;
            }

            if (this.layerData.urlError || this.layerData.nameError || this.layerData.typeError) {
                this.layerData.error = 'All Fields Are Required!';
                return;
            } else {
                this.layerData.error = false;
            }

            return error;
        },
        updateLayer: function(layer_idx) {
            if (isNaN(layer_idx)) return;
            if (this.validateLayer()) return;

            this.layers[layer_idx].name = this.layerData.name;
            this.layers[layer_idx].type = this.layerData.type;
            this.layers[layer_idx].url = this.layerData.url;

            this.putLayers();
            this.close();
        },
        clearLayer: function() {
            this.layerData.id = false;
            this.layerData.error = false;
            this.layerData.name = '';
            this.layerData.type = '';
            this.layerData.url = '';
        },
        addLayer: function() {
            if (this.validateLayer()) return;

            this.layers.push({
                name: this.layerData.name,
                type: this.layerData.type,
                url: this.layerData.url
            });

            this.putLayers();
            this.close();
        },
        editLayerClick: function(layer_idx) {
            if (isNaN(layer_idx)) return;

            this.mode = 'addLayer';
            this.layerData.id = layer_idx;
            this.layerData.name = this.layers[layer_idx].name;
            this.layerData.url = this.layers[layer_idx].url;
            this.layerData.type = this.layers[layer_idx].type;
        },
        newLayerClick: function() {
            this.mode = 'addLayer';
        },
        addHookClick: function() {
            this.mode = 'addHook';

            this.webhookData.id = false;
            this.webhookData.error = '';
            this.webhookData.name = '';
            this.webhookData.nameError = false;
            this.webhookData.url = '';
            this.webhookData.urlError = false;
            this.webhookData.actions = [];

            for (let check of Object.keys(this.webhookData.checkbox)) {
                this.webhookData.checkbox[check] = false;
            }
        },
        helpBaseClick: function() {
            this.mode = 'helpBase';
        },
        delLayerClick: function(layer_idx) {
            if (isNaN(Number(layer_idx))) return;

            this.mode = 'delLayer';

            this.delLayerData.idx = Number(layer_idx);
            this.delLayerData.name = this.layers[layer_idx].name;
            this.delLayerData.type = this.layers[layer_idx].type;
        }
    },
    render: h => h(App),
    props: ['auth']
}
</script>
