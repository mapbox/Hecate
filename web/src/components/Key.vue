<template>
    <div>
        <div class="w-full txt-s" style="text-align: center;">Version: <span v-text="feature.version"></span></div>
        <table class='table txt-xs'>
            <thead><tr><th>Key</th><th>Value</th></tr></thead>
            <tbody>
                <tr v-for="prop in Object.keys(feature.properties)">
                    <template v-if='schema && schema.properties && schema.properties[prop] && schema.properties[prop].description'>
                        <td @click="popup(schema.properties[prop].description)" class="cursor-pointer txt-underline-on-hover" v-text="prop">
                    </template>
                    <template v-else>
                        <td v-text="prop">
                    </template>

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
                            <!-- element: Array: (String) -->
                            <template v-else>
                                <td v-text="element"></td>
                            </template>
                            <!-- element: Array: (Array, Number) -->
                            <template v-else>
                                <td v-text="JSON.stringify(element)"></td>
                            </template>

                            <div style="border-bottom: solid #CBCBCB 1px;"></div>
                        </template>
                    </td>
                    <!-- element: (String) -->
                    <td v-else-if="typeof feature.properties[prop] === 'string'" v-text="feature.properties[prop]"></td>
                    <!-- element: (Object, Number) -->
                    <td v-else v-text="JSON.stringify(feature.properties[prop])"></td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<script>
export default {
    name: 'key',
    render: h => h(App),
    methods: {
        popup: function(description) {
            this.$emit('error', description);
        }
    },
    props: ['feature', 'schema']
}
</script>
