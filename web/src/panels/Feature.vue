<template>
<div class='flex-parent flex-parent--column viewport-third h-auto-ml bg-white round-ml shadow-darken10' style="pointer-events:auto; max-height: calc(100% - 80px);">
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold'>Feature <span v-text='feature.id'></span></h3>
        <button @click='close()' class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-close'/></button>
    </div>

    <template v-if='loading'>
        <div class='flex-child loading h60'></div>
    </template>
    <template v-else>
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
    </template>

    <foot/>
</div>
</template>

<script>
export default {
    name: 'feature',
    data: function() {
        return {
            feature: false,
            loading: false
        }
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
        get: function(id) {
            if (!id) return;
            if (typeof id === 'number' || typeof id === 'string') {
                this.loading = true;
            }

            fetch(`http://${window.location.host}/api/data/feature/${id}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                  return response.json();
            }).then((body) => {
                this.feature = body;
                this.loading = false;
            });
        }
    },
    render: h => h(App),
    props: ['id', 'map']
}
</script>
