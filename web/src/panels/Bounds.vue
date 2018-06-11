<template>
<div class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold'>Bounds</h3>
    </div>

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
</template>

<script>
export default {
    name: 'bounds',
    data: function() {
        bounds: []
    },
    created: function() {
        this.getBounds();
    },
    methods: {
        getBounds: function() {
            fetch(`http://${window.location.host}/api/data/bounds`).then((response) => {
                  return response.json();
            }).then((body) => {
                this.bounds = body;
            });
        }
    },
    render: h => h(App),
    props: []
}
</script>
