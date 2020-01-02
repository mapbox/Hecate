<template>
<div class='flex-parent flex-parent--column viewport-third h-auto-ml bg-white round-ml shadow-darken10' style="pointer-events:auto; max-height: calc(100% - 80px);">
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold'>Feature <span v-text='feature ? feature.id : ""'></span></h3>

        <template v-if="features.length && feature">
            <button @click='feature = false' class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-arrow-left'/></button>
        </template>
        <template v-else>
            <button @click='close' class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-close'/></button>
        </template>
    </div>

    <template v-if='is404'>
        <div class='flex-child h32 align-center py12'>
            No Feature Found
        </div>
    </template>
    <template v-else-if='feature'>
        <div class="flex-child scroll-auto">
            <template v-if='Array.isArray(feature)'>
                <template v-for="historic_feature in feature">
                    <key class='my12' :feature='historic_feature.feat' :schema="schema" v-on:error="description($event)"/>
                </template>
            </template>
            <template v-else>
                <key :feature='feature' :schema="schema" v-on:error="description($event)"/>

                <div class="w-full align-center my12">
                    <button @click='history' class='btn btn--s round bg-gray-light bg-darken25-on-hover color-gray-dark'>Load History</button>
                </div>
            </template>
        </div>
    </template>
    <template v-else-if='features.length'>
        <div class="flex-child scroll-auto">
            <div v-for="(feat, feat_it) in features" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                <div @click="feature = feat" class="w-full clearfix">
                    <span class="fl py6 px6"><svg class='icon'><use href='#icon-marker'/></span>
                    Feature # <span v-text="feat.id"></span>
            </div>
        </div>
    </template>
    <template v-else>
        <div class='flex-child loading h60'></div>
    </template>

    <foot/>
</div>
</template>

<script>
import Foot from '../components/Foot.vue';
import Key from '../components/Key.vue';

export default {
    name: 'feature',
    data: function() {
        return {
            is404: false,
            feature: false,
            features: []
        }
    },
    components: {
        foot: Foot,
        key: Key
    },
    watch: {
        id: function() {
            this.get(this.id);
        }
    },
    created: function() {
        this.get(this.id);
    },
    methods: {
        close: function() {
            this.$emit('close');
        },
        description: function(text) {
            this.$emit('error', {
                title: 'Property Details',
                body: text
            });
        },
        history: function() {
            window.hecate.feature.history(this.feature.id, (err, history) => {
                if (err) return this.$emit('error', err);
                history = history.map((feat, ele) => {
                    feat.feat.version = ele + 1;
                    return feat;
                });
                history.reverse();
                this.feature = history;
            });
        },
        get: function(id) {
            if (!id) return;

            this.feature = false;
            this.features = [];

            this.is404 = false;

            if (typeof id === 'number' || typeof id === 'string') {
                window.hecate.feature.get(id, (err, feature) => {
                    if (err) return this.$emit('error', err);
                    
                    if (!feature) {
                        this.is404 = true;
                    } else {
                        this.feature = feature;
                    }
                });
            } else if (Array.isArray(id)) {
                window.hecate.features.point(id, (err, features) => {
                    if (err) return this.$emit('error', err);
                    
                    if (!features) {
                        this.is404 = true;
                    } else if (features.length === 1) {
                        this.feature = features[0];
                    } else {
                        this.features = features;
                    }
                });

            }
        }
    },
    render: h => h(App),
    props: ['id', 'map', 'schema']
}
</script>
