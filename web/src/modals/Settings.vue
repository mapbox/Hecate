<template>
    <div class='absolute top left bottom right z3' style="pointer-events: none;">
        <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events: auto;">
            <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
                <template v-if='mode === "addLayer"'>
                    <div class='grid w-full col'>

                        <template v-if='addLayerData.error'>
                            <div class='col--12 color-white px12 bg-red round align-center'>
                                <h3 class='w-full py6 txt-m txt-bold' v-text='addLayerData.error'></h3>
                            </div>
                        </template>

                        <div class='col--12'>
                            <h3 class='w-full py6 txt-m txt-bold'>Add A New Base Layer</h3>
                        </div>

                        <div class='col--12 py12 px12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'>
                                    <label>Layer Name</label>
                                    <input v-model='addLayerData.name' class='input' placeholder='Layer Name' v-bind:class="{ 'input--border-red': addLayerData.nameError }"/>
                                </div>
                                <div class='col col--6'>
                                    <label >Layer Type</label>
                                    <div class='select-container w-full'>
                                        <select v-model='addLayerData.type' class='select' v-bind:class="{ 'input--border-red': addLayerData.typeError }">
                                            <option>Vector</option>
                                            <option>Raster</option>
                                        </select>
                                        <div class='select-arrow'></div>
                                    </div>
                                </div>
                                <div class='col col--12 py12'>
                                    <label>Mapbox:// Style</label>
                                    <input v-model='addLayerData.url' class='input w-full' placeholder='mapbox://' v-bind:class="{ 'input--border-red': addLayerData.urlError }" />
                                </div>
                            </div>
                        </div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'>
                                    <button @click='close' class='btn btn--red round w-full'>Cancel</button>
                                </div>
                                <div class='col col--6'>
                                    <button @click='addLayer' class='btn round w-full'>OK</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else-if='mode === "delLayer"'>
                    <div class='grid w-full col'>
                        <div class='col--12'>
                            <h3 class='w-full py6 txt-m txt-bold'>Are you sure?</h3>
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
                    <div class='grid w-full'>
                        <div class='col col--12'>
                            <h3 class='col col--12 fl py6 txt-m txt-bold'>Settings</h3>

                            <h4 class='fl py6 txt-m w-full border--gray-light border-b'>Base Layers</h4>

                            <div class='col col--12 hmin120 hmax180 clearfix'>
                                <template v-for='layer of layers'>
                                    <div class='h120 w120 fl relative color-gray-light my12 mx12 cursor-pointer'>
                                        <div @click='delLayer' class='absolute top right bg-red round color-white w18 h18'>
                                            <svg class='icon w-full pt3'><use xlink:href='#icon-close'/></svg>
                                        </div>

                                        <div class='w-full h-full color-gray-on-hover round border border--gray-light border--gray-on-hover'>
                                            <svg class='icon w-full h-full'><use href='#icon-paint'/></svg>
                                        </div>
                                    <div>
                                </template>

                                <!-- Add a new Base Layer -->
                                <div @click='newLayer' class='h120 w120 fl round border border--gray-light border--gray-on-hover color-gray-light color-gray-on-hover my12 mx12 cursor-pointer'>
                                    <svg class='icon w-full h-full'><use href='#icon-plus'/></svg>
                                </div>
                            </div>

                            <h4 class='fl py6 txt-m w-full border--gray-light border-b'>Default Style</h4>

                        </div>

                        <div class='col col--12'>
                            <button @click='close' class='fr mt24 btn round w-full'>OK</button>
                        </div>
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
            layers: [],
            addLayerData: {
                error: '',
                name: '',
                nameError: false,
                type: '',
                typeError: false,
                url: '',
                urlError: false
            }
        }
    },
    methods: {
        close: function() {
            if (this.mode !== 'settings') {
                this.mode = 'settings';
            } else {
                this.$emit('close');
            }
        },
        addLayer: function() {
            if (this.addLayerData.name.length === 0) {
                this.addLayerData.nameError = true;
            } else {
                this.addLayerData.nameError = false;
            }

            if (['Vector', 'Raster'].indexOf(this.addLayerData.type) === -1) {
                this.addLayerData.typeError = true;
            } else {
                this.addLayerData.typeError = false;
            }

            if (!this.addLayerData.url.match(/^mapbox:\/\//)) {
                this.addLayerData.urlError = true;
            } else {
                this.addLayerData.urlError = false;
            }

            if (this.addLayerData.urlError || this.addLayerData.nameError || this.addLayerData.typeError) {
                this.addLayerData.error = 'All Fields Are Required!';
                return;
            } else {
                this.addLayerData.error = false;
            }

            this.layers.push({
                name: this.addLayerData.name,
                type: this.addLayerData.type,
                url: this.addLayerData.url
            });

            this.addLayerData.error = false;
            this.addLayerData.name = '';
            this.addLayerData.type = '';
            this.addLayerData.url = '';

            this.close();
        },
        newLayer: function() {
            this.mode = 'addLayer';
        },
        delLayer: function() {
            this.mode = 'delLayer';
        }
    },
    render: h => h(App),
    props: []
}
</script>
