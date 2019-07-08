<template>
<div class='flex-parent flex-parent--column viewport-third h-auto-ml bg-white round-ml shadow-darken10' style="pointer-events:auto; max-height: calc(100% - 80px);">
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold'>Bounds</h3>
        <button @click="getBounds" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>
        <button @click="filterToggle = !filterToggle" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr mx3'><svg class='icon'><use href='#icon-search'/></button>
    </div>

    <div class="flex-child scroll-auto">
        <div v-if="filterToggle" class="relative px12 py3 clearfix bg-white">
            <div class='absolute flex-parent flex-parent--center-cross flex-parent--center-main w36 h36'>
                <svg class='icon'><use href='#icon-search'></use></svg>
            </div>
            <input class='input pl36' placeholder='Search' v-model="filter">
        </div>

        <template v-if='loading'>
            <div class='flex-child loading h60 py12'></div>
        </template>
        <template v-else>
            <div v-if="!bounds.length" class="px12 py3 clearfix bg-white">
                <div align="center">No Boundaries</div>
            </div>
            <div v-else>
                <div v-for="(bound_name, bound_it) in bounds" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                    <div @click="bound = bound_name" class="w-full clearfix">
                        <span class="fl py6 px6"><svg class='icon'><use href='#icon-database'/></span>
                        <span class="fl" v-text="bound_name"></span>
                        <a v-bind:href="`/api/data/bounds/${bound_name}`" target="_blank">
                            <button @click="getBounds" class='btn btn--stroke btn--s my6 mr6 round bg-transparent bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-arrow-down'/></button>
                        </a>
                    </div>
                </div>
            </div>
        </template>
    </div>

    <foot/>
</div>
</template>

<script>
import Foot from '../components/Foot.vue';
import { bbox } from '@turf/turf';

export default {
    name: 'bounds',
    data: function() {
        return {
            loading: true,
            filter: '',
            filterToggle: false,
            bounds: [],
            bound: false
        }
    },
    created: function() {
        this.getBounds();
    },
    watch: {
        filter: function() {
            this.getBounds();
        },
        bound: function() {
            this.showBounds();
        }
    },
    components: {
        foot: Foot
    },
    methods: {
        showBounds: function() {
            this.loading = true;

            fetch(`${window.location.protocol}//${window.location.host}/api/data/bounds/${this.bound}/meta`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                  return response.json();
            }).then((body) => {
                this.loading = false;

                this.map.gl.getSource('hecate-bounds').setData(body);
                this.map.gl.fitBounds(bbox(body));
            });

        },
        getBounds: function() {
            this.loading = true;

            let filter = '';
            if (this.filter) {
                filter = `&filter=${encodeURIComponent(this.filter)}`
            }

            fetch(`${window.location.protocol}//${window.location.host}/api/data/bounds?limit=100&${filter}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                  return response.json();
            }).then((body) => {
                this.bounds = body;
                this.loading = false;
            });
        }
    },
    render: h => h(App),
    props: [ 'map', 'panel' ]
}
</script>
