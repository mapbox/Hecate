<template>
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
</template>

<script>
export default {
    name: 'feature',
    render: h => h(App),
    props: ['feature']
}
</script>
